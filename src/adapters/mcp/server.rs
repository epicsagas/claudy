use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::Path;

use serde_json::{Value, json};

use crate::adapters::mcp::discovery;
use crate::adapters::mcp::runner;
use crate::config::registry::AppRegistry;
use crate::domain::agent::AgentDefinition;
use llm_kernel::mcp::{McpServer, ToolDescription};

pub fn run_mcp_server(config_path: &Path) -> anyhow::Result<i32> {
    let cfg = AppRegistry::open(config_path)?;
    let agents = discovery::discover_agents(&cfg.agents);

    tracing::info!(count = agents.len(), "Discovered agents for MCP server");

    ensure_registered_global();

    let server = build_server(&agents);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { server_loop(&server, &agents).await })?;

    Ok(0)
}

fn build_server(agents: &[AgentDefinition]) -> McpServer {
    let tool = build_ask_agent_tool(agents);
    let mut server = McpServer::new("claudy-mcp", env!("CARGO_PKG_VERSION"));
    server.register_tool(tool);
    server
}

fn build_ask_agent_tool(agents: &[AgentDefinition]) -> ToolDescription {
    let agent_list: Vec<String> = agents
        .iter()
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

    ToolDescription {
        name: "ask_agent".to_string(),
        description,
        input_schema: json!({
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
        }),
    }
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

async fn server_loop(server: &McpServer, agents: &[AgentDefinition]) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let agent_map: HashMap<&str, &AgentDefinition> =
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
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                writeln!(stdout, "{}", resp)?;
                stdout.flush()?;
                continue;
            }
        };

        let id = msg.get("id").cloned().unwrap_or(Value::Null);

        // JSON-RPC 2.0: servers MUST NOT reply to notifications (requests without an id).
        if msg.get("id").is_none() {
            continue;
        }

        let method = msg["method"].as_str().unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": server.initialize_response()
            }),
            "tools/list" => {
                // llm-kernel's ToolDescription serializes `input_schema` in
                // snake_case, but the MCP spec (2024-11-05) requires
                // `inputSchema`. Emit each tool explicitly so clients accept it.
                let tools: Vec<Value> = server
                    .tools()
                    .iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.input_schema,
                        })
                    })
                    .collect();
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "tools": tools }
                })
            }
            "tools/call" => handle_tools_call(&id, &params, &agent_map).await,
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("Method not found: {}", method) }
            }),
        };

        writeln!(stdout, "{}", response)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_tools_call(
    id: &Value,
    params: &Value,
    agents: &HashMap<&str, &AgentDefinition>,
) -> Value {
    let args = match params.get("arguments") {
        Some(a) => a,
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Missing arguments" }
            });
        }
    };

    let agent_name = match args["agent"].as_str() {
        Some(n) => n,
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Missing 'agent' parameter" }
            });
        }
    };

    let prompt = match args["prompt"].as_str() {
        Some(p) if !p.is_empty() => p,
        _ => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "Missing 'prompt' parameter" }
            });
        }
    };

    if prompt.len() > 100_000 {
        return json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32602, "message": "Prompt exceeds maximum length of 100,000 characters" }
        });
    }

    let cwd = args["working_directory"].as_str().map(|s| s.to_string());

    let def = match agents.get(agent_name) {
        Some(d) => (*d).clone(),
        None => {
            let available: Vec<&str> = agents.keys().copied().collect();
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "content": [{
                        "type": "text",
                        "text": format!("Agent '{}' is not installed. Available: {}", agent_name, available.join(", "))
                    }],
                    "isError": true
                }
            });
        }
    };

    match runner::run_agent(&def, prompt, cwd.as_deref().map(Path::new)).await {
        Ok(output) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": output.trim()
                }]
            }
        }),
        Err(e) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": format!("Agent '{}' failed: {}", agent_name, e)
                }],
                "isError": true
            }
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // ── build_server / build_ask_agent_tool ───────────────────────────

    #[test]
    fn test_build_server_initialize_response() {
        let server = build_server(&[]);
        let resp = server.initialize_response();

        assert_eq!(resp["protocolVersion"], "2024-11-05");
        assert!(resp["capabilities"]["tools"].is_object());
        assert_eq!(resp["serverInfo"]["name"], "claudy-mcp");
    }

    #[test]
    fn test_build_ask_agent_tool_with_agents() {
        let agent = AgentDefinition {
            name: "test-agent".into(),
            binary: "echo".into(),
            args: vec![],
            description: "Test agent".into(),
            timeout: 10,
        };
        let agents = vec![agent];
        let tool = build_ask_agent_tool(&agents);

        assert_eq!(tool.name, "ask_agent");
        assert!(tool.description.contains("test-agent"));
    }

    #[test]
    fn test_build_ask_agent_tool_empty() {
        let tool = build_ask_agent_tool(&[]);
        assert!(
            tool.description
                .contains("No agents are currently installed")
        );
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
        assert!(settings["mcpServers"]["claudy"].is_object());
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
        unregister(&path);
    }

    #[test]
    fn test_ensure_registered_does_not_overwrite_corrupted_json() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("settings.json");

        let corrupted = r"{ this is not valid JSON !!!";
        std::fs::write(&path, corrupted).unwrap();

        ensure_registered(&path);

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

        let metadata_before = std::fs::metadata(&path).unwrap();
        let modified_before = metadata_before.modified().unwrap();

        ensure_registered(&path);

        let content_after = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content_after, original);

        let metadata_after = std::fs::metadata(&path).unwrap();
        let modified_after = metadata_after.modified().unwrap();
        assert_eq!(
            modified_before, modified_after,
            "File was rewritten despite matching entry"
        );
    }
}
