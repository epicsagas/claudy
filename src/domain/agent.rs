use serde::{Deserialize, Serialize};

/// A resolved agent definition ready for execution.
#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub binary: String,
    /// Argument template. Use `"{prompt}"` as a placeholder for the user prompt.
    pub args: Vec<String>,
    pub description: String,
    /// Timeout in seconds. Defaults to 120.
    pub timeout: u64,
}

/// User-configurable agent override from config.yaml.
/// All fields are optional — only the fields present override the built-in defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

/// Built-in agent catalog.
/// Each entry maps to a known coding agent CLI with its headless command template.
pub fn builtin_agents() -> Vec<AgentDefinition> {
    vec![
        // Tier 1: Major provider native
        AgentDefinition {
            name: "codex".into(),
            binary: "codex".into(),
            args: vec!["exec".into(), "{prompt}".into()],
            description: "OpenAI Codex CLI (ChatGPT subscription)".into(),
            timeout: 3600,
        },
        AgentDefinition {
            name: "copilot".into(),
            binary: "copilot".into(),
            args: vec!["-p".into(), "{prompt}".into()],
            description: "GitHub Copilot CLI".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "agent".into(),
            binary: "agent".into(),
            args: vec![
                "-p".into(),
                "{prompt}".into(),
                "--output-format".into(),
                "text".into(),
            ],
            description: "Cursor Agent CLI".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "agy".into(),
            binary: "agy".into(),
            args: vec!["-p".into(), "{prompt}".into()],
            description: "Antigravity CLI (agy)".into(),
            timeout: 120,
        },
        // Tier 2: Open source / Independent
        AgentDefinition {
            name: "opencode".into(),
            binary: "opencode".into(),
            args: vec!["run".into(), "{prompt}".into()],
            description: "OpenCode (75+ providers, 140K+ stars)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "cline".into(),
            binary: "cline".into(),
            args: vec!["-y".into(), "{prompt}".into()],
            description: "Cline CLI (autonomous coding agent)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "goose".into(),
            binary: "goose".into(),
            args: vec!["run".into(), "{prompt}".into()],
            description: "Goose (Block/Square, Apache 2.0)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "amp".into(),
            binary: "amp".into(),
            args: vec!["--non-interactive".into(), "{prompt}".into()],
            description: "Amp (Sourcegraph, deep codebase analysis)".into(),
            timeout: 180,
        },
        // Tier 3: Specialized / New
        AgentDefinition {
            name: "droid".into(),
            binary: "droid".into(),
            args: vec!["exec".into(), "{prompt}".into()],
            description: "Droid (Factory AI, specialized sub-agents)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "kiro".into(),
            binary: "kiro-cli".into(),
            args: vec![
                "chat".into(),
                "--no-interactive".into(),
                "--trust-all-tools".into(),
                "{prompt}".into(),
            ],
            description: "Kiro CLI (AWS, spec-driven development)".into(),
            timeout: 180,
        },
        AgentDefinition {
            name: "junie".into(),
            binary: "junie".into(),
            args: vec!["{prompt}".into()],
            description: "Junie CLI (JetBrains, LLM-agnostic)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "kimi".into(),
            binary: "kimi".into(),
            args: vec!["{prompt}".into()],
            description: "Kimi Code CLI (Moonshot K2.5)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "vibe".into(),
            binary: "vibe".into(),
            args: vec!["{prompt}".into()],
            description: "Mistral Vibe (Apache 2.0)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "qwen-code".into(),
            binary: "qwen-code".into(),
            args: vec!["{prompt}".into()],
            description: "Qwen Code CLI (Alibaba, free API)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "crush".into(),
            binary: "crush".into(),
            args: vec!["{prompt}".into()],
            description: "Crush (Charmbracelet, cross-platform)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "groq-code".into(),
            binary: "groq-code".into(),
            args: vec!["--prompt".into(), "{prompt}".into()],
            description: "Groq Code CLI (ultra-low latency)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "plandex".into(),
            binary: "plandex".into(),
            args: vec!["tell".into(), "{prompt}".into()],
            description: "Plandex (large project planning)".into(),
            timeout: 180,
        },
        AgentDefinition {
            name: "kilo".into(),
            binary: "kilo".into(),
            args: vec!["{prompt}".into()],
            description: "Kilo Code (500+ models, 60+ providers)".into(),
            timeout: 120,
        },
        AgentDefinition {
            name: "openhands".into(),
            binary: "openhands".into(),
            args: vec!["{prompt}".into()],
            description: "OpenHands CLI (lightweight standalone)".into(),
            timeout: 120,
        },
    ]
}
