use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::Path;

use serde_json::{Value, json};

use crate::adapters::mcp::discovery;
use crate::adapters::mcp::runner;
use crate::config::registry::AppRegistry;

/// Build a JSON-RPC 2.0 success response.
fn rpc_response(id: &Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

/// Build a JSON-RPC 2.0 error response.
fn rpc_error(id: &Value, code: i32, message: impl std::fmt::Display) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message.to_string() }
    })
}

pub fn run_mcp_server(config_path: &Path) -> anyhow::Result<i32> {
    let cfg = AppRegistry::open(config_path)?;
    let agents = discovery::discover_agents(&cfg.agents);

    tracing::info!(count = agents.len(), "Discovered agents for MCP server");

    ensure_registered_global();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { server_loop(&agents).await })?;

    Ok(0)
}

/// The expected claudy MCP server entry.
fn claudy_entry() -> Value {
    json!({
        "command": "claudy",
        "args": ["mcp", "run"]
    })
}

/// Check whether the existing claudy entry already matches the expected one.
fn entry_matches(existing: &Value, expected: &Value) -> bool {
    existing.get("command") == expected.get("command")
        && existing.get("args") == expected.get("args")
}

/// Ensure the claudy MCP server is registered in the given settings file.
/// Upsert — writes the claudy entry only when missing or changed (idempotent).
pub fn ensure_registered(settings_path: &Path) {
    let dir = settings_path.parent();
    if let Some(d) = dir {
        let _ = std::fs::create_dir_all(d);
    }

    let mut settings: Value = match std::fs::read_to_string(settings_path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    path = %settings_path.display(),
                    error = %e,
                    "Settings file contains malformed JSON; refusing to overwrite"
                );
                return;
            }
        },
        Err(_) => json!({}),
    };

    let expected = claudy_entry();

    // Idempotent: skip the write if the entry already has the correct value.
    if settings
        .get("mcpServers")
        .and_then(|m| m.get("claudy"))
        .is_some_and(|existing| entry_matches(existing, &expected))
    {
        return;
    }

    if settings.get("mcpServers").is_none() {
        settings["mcpServers"] = json!({});
    }
    settings["mcpServers"]["claudy"] = expected;

    match serde_json::to_string_pretty(&settings) {
        Ok(pretty) => {
            let data = format!("{}\n", pretty);
            if let Err(e) = crate::config::atomic::write_atomic(
                &settings_path.to_string_lossy(),
                data.as_bytes(),
                0o644,
            ) {
                tracing::warn!(path = %settings_path.display(), error = %e, "Failed to write settings file");
            } else {
                tracing::info!(path = %settings_path.display(), "Registered claudy MCP server");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to serialize settings");
        }
    }
}

/// Register in global Claude Code settings: ~/.claude.json
pub fn ensure_registered_global() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    ensure_registered(&home.join(".claude.json"));
}

/// Register in a specific mode's settings: <modes_dir>/<name>/.claude.json
pub fn ensure_registered_mode(modes_dir: &str, mode_name: &str) {
    let path = Path::new(modes_dir).join(mode_name).join(".claude.json");
    ensure_registered(&path);
}

/// Remove the claudy MCP entry from the given settings file.
/// No-op if the file or entry doesn't exist. Refuses to touch a malformed file.
pub fn unregister(settings_path: &Path) {
    let mut settings: Value = match std::fs::read_to_string(settings_path) {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(
                    path = %settings_path.display(),
                    error = %e,
                    "Settings file contains malformed JSON; refusing to overwrite"
                );
                return;
            }
        },
        Err(_) => return,
    };

    if settings
        .get("mcpServers")
        .and_then(|m| m.get("claudy"))
        .is_none()
    {
        return;
    }

    if let Some(servers) = settings
        .get_mut("mcpServers")
        .and_then(|m| m.as_object_mut())
    {
        servers.remove("claudy");
    }

    match serde_json::to_string_pretty(&settings) {
        Ok(pretty) => {
            let data = format!("{}\n", pretty);
            if let Err(e) = crate::config::atomic::write_atomic(
                &settings_path.to_string_lossy(),
                data.as_bytes(),
                0o644,
            ) {
                tracing::warn!(path = %settings_path.display(), error = %e, "Failed to write settings file");
            } else {
                tracing::info!(path = %settings_path.display(), "Unregistered claudy MCP server");
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to serialize settings");
        }
    }
}

/// Unregister from global Claude Code settings: ~/.claude.json
pub fn unregister_global() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    unregister(&home.join(".claude.json"));
}

async fn server_loop(agents: &[crate::domain::agent::AgentDefinition]) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let agent_map: HashMap<&str, &crate::domain::agent::AgentDefinition> =
        agents.iter().map(|a| (a.name.as_str(), a)).collect();

    let reader = stdin.lock();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let id = Value::Null;
                let resp = rpc_error(&id, -32700, format!("Parse error: {}", e));
                writeln!(stdout, "{}", resp)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        let method = msg["method"].as_str().unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        // JSON-RPC 2.0: servers MUST NOT reply to notifications (requests without an id).
        if msg.get("id").is_none() {
            continue;
        }

        let response = match method {
            "initialize" => handle_initialize(&id),
            // Per JSON-RPC 2.0, all notifications are skipped above; this arm is retained
            // as a documentation marker for the initialized handshake notification.
            "notifications/initialized" => {
                continue;
            }
            "notifications/cancelled" => {
                // Cancellation sent by the client during long-running calls; no response.
                continue;
            }
            "tools/list" => handle_tools_list(&id, &agent_map),
            "tools/call" => handle_tools_call(&id, &params, &agent_map).await,
            _ => rpc_error(&id, -32601, format!("Method not found: {}", method)),
        };

        writeln!(stdout, "{}", response)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_initialize(id: &Value) -> Value {
    rpc_response(
        id,
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "claudy-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(
    id: &Value,
    agents: &HashMap<&str, &crate::domain::agent::AgentDefinition>,
) -> Value {
    let agent_list: Vec<String> = agents
        .values()
        .map(|a| format!("- **{}**: {}", a.name, a.description))
        .collect();

    let description = if agent_list.is_empty() {
        "Delegate a task to a local AI coding agent. No agents are currently installed.".into()
    } else {
        format!(
            "Delegate a task to a local AI coding agent. Available agents:\n{}",
            agent_list.join("\n")
        )
    };

    rpc_response(
        id,
        json!({
            "tools": [{
                "name": "ask_agent",
                "description": description,
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "agent": {
                            "type": "string",
                            "description": "Agent name (from the list above)"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "The task or question for the agent"
                        },
                        "working_directory": {
                            "type": "string",
                            "description": "Working directory for the agent (optional)"
                        }
                    },
                    "required": ["agent", "prompt"]
                }
            }]
        }),
    )
}

async fn handle_tools_call(
    id: &Value,
    params: &Value,
    agents: &HashMap<&str, &crate::domain::agent::AgentDefinition>,
) -> Value {
    let args = match params.get("arguments") {
        Some(a) => a,
        None => {
            return rpc_error(id, -32602, "Missing arguments");
        }
    };

    let agent_name = match args["agent"].as_str() {
        Some(n) => n,
        None => {
            return rpc_error(id, -32602, "Missing 'agent' parameter");
        }
    };

    let prompt = match args["prompt"].as_str() {
        Some(p) if !p.is_empty() => p,
        _ => {
            return rpc_error(id, -32602, "Missing 'prompt' parameter");
        }
    };

    if prompt.len() > 100_000 {
        return rpc_error(
            id,
            -32602,
            "Prompt exceeds maximum length of 100,000 characters",
        );
    }

    let cwd = args["working_directory"].as_str().map(|s| s.to_string());

    let def = match agents.get(agent_name) {
        Some(d) => (*d).clone(),
        None => {
            let available: Vec<&str> = agents.keys().copied().collect();
            return rpc_response(
                id,
                json!({
                    "content": [{
                        "type": "text",
                        "text": format!("Agent '{}' is not installed. Available: {}", agent_name, available.join(", "))
                    }],
                    "isError": true
                }),
            );
        }
    };

    match runner::run_agent(&def, prompt, cwd.as_deref().map(Path::new)).await {
        Ok(output) => rpc_response(
            id,
            json!({
                "content": [{
                    "type": "text",
                    "text": output.trim()
                }]
            }),
        ),
        Err(e) => rpc_response(
            id,
            json!({
                "content": [{
                    "type": "text",
                    "text": format!("Agent '{}' failed: {}", agent_name, e)
                }],
                "isError": true
            }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::agent::AgentDefinition;
    use serde_json::json;
    use std::collections::HashMap;

    // ── handle_initialize ────────────────────────────────────────────

    #[test]
    fn test_handle_initialize() {
        let resp = handle_initialize(&json!(1));

        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["id"], 1);
        assert_eq!(resp["result"]["protocolVersion"], "2024-11-05");
        assert!(resp["result"]["capabilities"]["tools"].is_object());
        assert_eq!(resp["result"]["serverInfo"]["name"], "claudy-mcp");
    }

    // ── handle_tools_list ────────────────────────────────────────────

    #[test]
    fn test_handle_tools_list_with_agents() {
        let agent = AgentDefinition {
            name: "test-agent".into(),
            binary: "echo".into(),
            args: vec![],
            description: "Test agent".into(),
            timeout: 10,
        };
        let mut map: HashMap<&str, &AgentDefinition> = HashMap::new();
        map.insert("test-agent", &agent);

        let resp = handle_tools_list(&json!(2), &map);

        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "ask_agent");
        let desc = tools[0]["description"].as_str().unwrap();
        assert!(desc.contains("test-agent"));
        let required = tools[0]["inputSchema"]["required"].as_array().unwrap();
        assert!(required.iter().any(|r| r == "agent"));
        assert!(required.iter().any(|r| r == "prompt"));
    }

    #[test]
    fn test_handle_tools_list_empty() {
        let map: HashMap<&str, &AgentDefinition> = HashMap::new();
        let resp = handle_tools_list(&json!(3), &map);

        let desc = resp["result"]["tools"][0]["description"].as_str().unwrap();
        assert!(desc.contains("No agents are currently installed"));
    }

    // ── handle_tools_call (async) ────────────────────────────────────

    #[tokio::test]
    async fn test_handle_tools_call_missing_arguments() {
        let map: HashMap<&str, &AgentDefinition> = HashMap::new();
        let resp = handle_tools_call(&json!(4), &json!({}), &map).await;

        assert_eq!(resp["error"]["code"], -32602);
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Missing arguments")
        );
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_agent() {
        let map: HashMap<&str, &AgentDefinition> = HashMap::new();
        let resp = handle_tools_call(&json!(5), &json!({"arguments": {}}), &map).await;

        assert_eq!(resp["error"]["code"], -32602);
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Missing 'agent'")
        );
    }

    #[tokio::test]
    async fn test_handle_tools_call_missing_prompt() {
        let map: HashMap<&str, &AgentDefinition> = HashMap::new();
        let resp = handle_tools_call(&json!(6), &json!({"arguments": {"agent": "x"}}), &map).await;

        assert_eq!(resp["error"]["code"], -32602);
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Missing 'prompt'")
        );
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown_agent() {
        let map: HashMap<&str, &AgentDefinition> = HashMap::new();
        let params = json!({"arguments": {"agent": "nonexistent", "prompt": "hi"}});
        let resp = handle_tools_call(&json!(7), &params, &map).await;

        assert_eq!(resp["result"]["content"][0]["type"], "text");
        assert_eq!(resp["result"]["isError"], true);
        let text = resp["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("not installed"));
    }

    // ── ensure_registered ────────────────────────────────────────────

    #[test]
    fn test_ensure_registered_creates_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        ensure_registered(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["mcpServers"]["claudy"]["command"], "claudy");
    }

    #[test]
    fn test_ensure_registered_idempotent() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        ensure_registered(&path);
        ensure_registered(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        // There should be exactly one claudy entry
        assert!(settings["mcpServers"]["claudy"].is_object());
        // No duplicate keys possible in JSON, but verify the entry is stable
        assert_eq!(settings["mcpServers"]["claudy"]["command"], "claudy");
        assert_eq!(
            settings["mcpServers"]["claudy"]["args"],
            json!(["mcp", "run"])
        );
    }

    #[test]
    fn test_ensure_registered_preserves_existing() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let existing = json!({
            "mcpServers": {
                "other": { "command": "other" }
            }
        });
        std::fs::write(&path, serde_json::to_string(&existing).unwrap()).unwrap();

        ensure_registered(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["mcpServers"]["other"]["command"], "other");
        assert_eq!(settings["mcpServers"]["claudy"]["command"], "claudy");
    }

    #[test]
    fn test_unregister_removes_entry() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        ensure_registered(&path);
        unregister(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(settings["mcpServers"]["claudy"].is_null());
    }

    #[test]
    fn test_unregister_preserves_other_servers() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let existing = json!({
            "mcpServers": {
                "other": { "command": "other" },
                "claudy": { "command": "claudy", "args": ["mcp"] }
            }
        });
        std::fs::write(&path, serde_json::to_string(&existing).unwrap()).unwrap();

        unregister(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["mcpServers"]["other"]["command"], "other");
        assert!(settings["mcpServers"]["claudy"].is_null());
    }

    #[test]
    fn test_unregister_noop_when_missing() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let existing = json!({
            "mcpServers": {
                "other": { "command": "other" }
            }
        });
        std::fs::write(&path, serde_json::to_string(&existing).unwrap()).unwrap();

        unregister(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(settings["mcpServers"]["other"]["command"], "other");
    }

    #[test]
    fn test_unregister_noop_when_no_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("nonexistent").join("settings.json");
        // Should not panic
        unregister(&path);
    }

    // ── corrupted JSON protection ────────────────────────────────────

    #[test]
    fn test_ensure_registered_does_not_overwrite_corrupted_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let corrupted = r"{ this is not valid JSON !!!";
        std::fs::write(&path, corrupted).unwrap();

        ensure_registered(&path);

        // File must be left exactly as-is — NOT replaced with a fresh object.
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, corrupted);
    }

    #[test]
    fn test_unregister_does_not_overwrite_corrupted_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let corrupted = r"{ broken json content }";
        std::fs::write(&path, corrupted).unwrap();

        unregister(&path);

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, corrupted);
    }

    #[test]
    fn test_ensure_registered_skips_write_when_entry_matches() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        // Pre-populate with the exact entry ensure_registered would write.
        let existing = json!({
            "mcpServers": {
                "claudy": {
                    "command": "claudy",
                    "args": ["mcp", "run"]
                }
            }
        });
        let original = serde_json::to_string_pretty(&existing).unwrap() + "\n";
        std::fs::write(&path, &original).unwrap();

        // Snapshot the file metadata before calling ensure_registered.
        let metadata_before = std::fs::metadata(&path).unwrap();
        let modified_before = metadata_before.modified().unwrap();

        ensure_registered(&path);

        // The file content should be byte-for-byte identical (no rewrite).
        let content_after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, original);

        // On filesystems that support it, the modification time should not change.
        // (This is a soft check — some FS granularity may cause false passes.)
        let metadata_after = std::fs::metadata(&path).unwrap();
        let modified_after = metadata_after.modified().unwrap();
        assert_eq!(
            modified_before, modified_after,
            "File was rewritten despite matching entry"
        );
    }
}
