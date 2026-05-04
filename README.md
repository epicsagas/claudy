<p align="center">
  <a href="docs/i18n/README.ko.md">🇰🇷 한국어</a> •
  <a href="docs/i18n/README.zh-Hans.md">🇨🇳 中文</a> •
  <a href="docs/i18n/README.ja.md">🇯🇵 日本語</a> •
  <a href="docs/i18n/README.de.md">🇩🇪 Deutsch</a> •
  <a href="docs/i18n/README.fr.md">🇫🇷 Français</a> •
  <a href="docs/i18n/README.es.md">🇪🇸 Español</a> •
  <a href="docs/i18n/README.hi.md">🇮🇳 हिन्दी</a> •
  <a href="docs/i18n/README.pt-BR.md">🇧🇷 Português</a> •
  <a href="docs/i18n/README.id.md">🇮🇩 Bahasa</a> •
  <a href="docs/i18n/README.ar.md">🇸🇦 العربية</a>
</p>

<h1 align="center">claudy</h1>

<p align="center"><b>Modern multi-provider launcher for Claude CLI.</b></p>

---

<p align="center">
Claudy helps you run Claude against multiple providers with one consistent command surface, while keeping provider credentials and Claude config overlays organized under a single home directory.
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="assets/features-2048.png" alt="Why Claudy" width="100%" />

## Why Claudy

- **Multi-provider launch**: switch across built-in, Z.AI, OpenRouter alias, Ollama and custom Anthropic-compatible endpoints.
- **Config modes**: isolate Claude configuration (`CLAUDE.md`, `settings.json`, skills/plugins/agents) per mode.
- **Provider profile resolution**: unify built-in providers, custom providers, and OpenRouter aliases.
- **Safe process behavior**: forwards SIGINT/SIGTERM to child Claude process.
- **Operational UX**: install/update/uninstall commands, status checks, and connectivity tests.
- **Optional channel bridge**: run a local bot bridge for Telegram, Slack, and Discord with interactive permission prompts.
- **Agent MCP bridge**: delegate tasks from Claude Code to other local AI agents (Gemini, Codex, Aider, etc.) via MCP.
- **Usage analytics**: ingest session data from `~/.claude/projects/`, track token usage and costs per session/project, view a local dashboard with recommendations.

## Provider Status

> Claudy was inspired by [Clother](https://github.com/jolehuit/clother), a Go-based multi-provider launcher for Claude CLI. Only the **Z.AI provider has been fully tested**. All other alternative providers are experimental and untested — use them at your own risk.

| Provider | Status | Notes |
|---|---|---|
| Built-in (Anthropic) | ✅ Tested | Default |
| Z.AI | ✅ Tested | Fully validated |
| OpenRouter alias | ⚠️ Experimental | Untested — use at your own risk |
| Ollama | ⚠️ Experimental | Untested — use at your own risk |
| Custom endpoint | ⚠️ Experimental | Untested — use at your own risk |

## Requirements

- macOS or Linux
- Rust toolchain (`cargo`) for build/install from source
- Claude CLI installed and available in `PATH`

## Installation

### Install from crates.io

**Pre-built binary (fast, no compilation)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**Any platform — build from source**

```
cargo install claudy
```

**MacOS homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### Install from local source

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### Verify

```bash
claudy --help
claudy --version
```

## Quick Start

```bash
# 1) List available/resolved profiles
claudy ls

# 2) Configure credentials interactively
claudy setup

# 3) Check one profile's details
claudy show <profile>

# 4) Run Claude with a profile
claudy <profile> [claude-args...]
```

## Core Concepts

### Profile

A launch target that resolves provider metadata + auth strategy (built-in provider, OpenRouter alias, or custom provider).

### Mode

A named Claude config directory at `~/.claudy/modes/<name>/`.

When you run:

```bash
claudy <profile> <mode> [args...]
```

Claudy sets:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

so Claude reads mode-specific config files.

## Command Reference

### Main commands

- `claudy ls` (alias: `list`): list configured/resolved profiles.
- `claudy setup [provider]` (alias: `config`): interactive provider setup.
- `claudy show <profile>` (alias: `info`): show resolved provider details.
- `claudy ping [profile]` (alias: `test`): test provider connectivity.
- `claudy doctor` (alias: `status`): show version, paths, and profile count.
- `claudy sync` (alias: `install`): install/synchronize claudy binary.
- `claudy update`: update claudy.
- `claudy uninstall`: remove installed files.
- `claudy mode <action> [name]`: manage Claude config modes.
- `claudy channel <subcommand>`: manage channel bridge.
- `claudy mcp`: run as MCP server for agent bridge.
- `claudy analytics <subcommand>`: usage analytics dashboard.

### Mode commands

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Mode name rule: `[a-z0-9][a-z0-9_-]*` (`mode` is reserved).

### Channel commands (optional bridge)

```bash
claudy channel start [--profile <profile>] [--listen <host:port>]
claudy channel stop
claudy channel restart
claudy channel status
claudy channel add <telegram|slack|discord>
claudy channel remove <telegram|slack|discord>
claudy channel enable <telegram|slack|discord>
claudy channel disable <telegram|slack|discord>
```

`channel add` guides you through bot token, allowed users, profile, and mode mapping.

#### Supported platforms

| Platform | Ingestion | Interactive buttons | Notes |
|----------|-----------|-------------------|-------|
| Telegram | Long-polling + webhook | Inline keyboard | Most complete |
| Slack | Event subscription webhook | Block Kit actions | HMAC-SHA256 verified |
| Discord | Interaction webhook | Action row components | Ed25519 verified |

#### Channel bot commands

Once running, the bot responds to these commands in chat:

- `/help` — Show available commands
- `/cancel` — Cancel current task
- `/model` — Change Claude model (interactive buttons)
- `/yolo` — Toggle auto-allow permissions
- `/status` — Show session status, profile, mode, git branch, and token usage
- `/sessions` — List recent Claude sessions (with switch buttons)
- `/projects` — List projects (with browse buttons)
- `/new` — Start a new session
- `/history` — Show recent session history

Send any other text to talk directly to Claude.

#### Permission prompts

When Claude requests approval to use a tool (run a command, edit a file, etc.),
the bot sends an interactive Allow/Deny prompt to your chat. Tapping a button
sends the response back to Claude and processing continues automatically.

#### Secrets

Store credentials in `~/.claudy/secrets.env`:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP bridge

Run `claudy mcp` to start a stdio-based MCP server that lets Claude Code delegate tasks to other locally installed AI coding agents.

```bash
claudy mcp
```

On first run, claudy automatically registers itself in `~/.claude/settings.json`. When you create a mode with `claudy mode create <name>`, it also registers in the mode's settings file. No manual configuration needed.

To register manually (or in a project-level `.claude/settings.json`):

```json
{
  "mcpServers": {
    "claudy": {
      "command": "claudy",
      "args": ["mcp"]
    }
  }
}
```

Claude Code will see an `ask_agent` tool that exposes all installed agents.

#### Usage example

Once registered, Claude Code can delegate tasks like this:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code selects the appropriate agent, passes the prompt, and returns the result. You can also specify a working directory:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### Verify MCP registration

```bash
# Check if claudy is registered
cat ~/.claude/settings.json | grep -A3 claudy

# Test the MCP server manually
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### Supported agents (auto-detected from PATH)

| Agent | Binary | Headless command |
|-------|--------|-----------------|
| Gemini CLI | `gemini` | `gemini -p "..." --output-format text` |
| Codex CLI | `codex` | `codex exec "..."` |
| Cursor Agent | `agent` | `agent -p "..." --output-format text` |
| GitHub Copilot | `copilot` | `copilot -p "..."` |
| OpenCode | `opencode` | `opencode run "..."` |
| Cline | `cline` | `cline -y "..."` |
| Aider | `aider` | `aider --message "..."` |
| Goose | `goose` | `goose run "..."` |
| Amp | `amp` | `amp --non-interactive "..."` |
| Droid | `droid` | `droid exec "..."` |
| Kiro | `kiro-cli` | `kiro-cli chat --no-interactive --trust-all-tools "..."` |
| Junie | `junie` | `junie "..."` |
| Kimi Code | `kimi` | `kimi "..."` |
| Mistral Vibe | `vibe` | `vibe "..."` |
| Qwen Code | `qwen-code` | `qwen-code "..."` |
| Crush | `crush` | `crush "..."` |
| Groq Code | `groq-code` | `groq-code --prompt "..."` |
| Plandex | `plandex` | `plandex tell "..."` |
| Kilo Code | `kilo` | `kilo "..."` |
| OpenHands | `openhands` | `openhands "..."` |

#### Custom agents

Add agents in `~/.claudy/config.json`:

```json
{
  "agents": {
    "my-agent": {
      "binary": "my-agent",
      "args": ["--prompt", "{prompt}", "--no-interactive"],
      "description": "My custom agent",
      "timeout": 180
    }
  }
}
```

Same key as a built-in agent overrides its defaults. `{prompt}` in `args` is replaced with the actual task.

### Analytics commands

> **Note**: The analytics feature is still a work in progress. Token counts, cost estimates, and other metrics may not be fully accurate. Expect refinements in upcoming releases.

```bash
claudy analytics dashboard         # Open local analytics dashboard (Tauri 2)
claudy analytics ingest            # Ingest session data from ~/.claude/projects/
claudy analytics ingest --full     # Re-ingest all files (ignore checkpoints)
claudy analytics ingest --project my-project  # Ingest specific project
claudy analytics recommend         # Show usage recommendations in CLI
claudy analytics export            # Export analytics data (JSON, default 30 days)
claudy analytics export --format csv --days 7  # Export as CSV for last 7 days
```

Analytics tracks:

- **Tokens**: Detailed trends of input, output, and cache tokens over the last 30 days, grouped by model and date.
- **Tools**: Distribution analysis showing which tools Claude uses most frequently, including call counts, error rates, and average execution time.
- **Cost**: Real-time estimation of usage costs based on actual token pricing, including daily/weekly/monthly forecasts and trend detection (increasing/stable/decreasing).
- **Tips (Recommendations)**: Data-driven optimization advice, such as detecting high-cost sessions, suggesting Haiku for simple tasks, and identifying long conversations that could benefit from context summarization.
- **Projects**: Automatically maps cryptic session UUIDs to human-readable project folder names for better context.

Data is stored in a local SQLite database under `~/.claudy/analytics/`. The dashboard runs as a high-performance local Tauri 2 + Svelte app. Use the **[Sync]** button in the dashboard to instantly refresh data from your Claude CLI history.

<img src="assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## Files and Directory Layout

By default, Claudy stores data under:

```text
~/.claudy/
```

Important files/directories:

- `config.json`: provider + channel + agent configuration.
- `secrets.env`: provider/bot credentials.
- `launchers.json`: launcher/symlink manifest.
- `modes/`: Claude config modes.
- `session-patches/`: session patch storage.
- `channel/`: channel runtime state (`pid`, sessions, audit log).
- `analytics/`: analytics SQLite database and checkpoints.
- `cache/update.json`: update metadata cache.

## Environment Variables

- `CLAUDY_HOME`: override the Claudy home directory (default: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: set automatically by Claudy when launching with a mode.

## Common Workflows

### Configure and launch a provider

```bash
claudy setup
claudy <profile>
```

### Use a mode with a provider

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` is claudy's shorthand for `--dangerously-skip-permissions`.

### Delegate tasks to other agents via MCP

```bash
# 1) Ensure MCP is registered (happens automatically on first `claudy mcp`)
claudy mcp

# 2) In Claude Code, ask it to delegate to any installed agent:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnose install/configuration state

```bash
claudy doctor
claudy ping
```

## Troubleshooting

- **`profile not recognized`**: run `claudy ls` and choose a listed profile ID.
- **`not configured` profile**: run `claudy setup <provider>` to add credentials.
- **Channel status unhealthy**: run `claudy channel status`, then restart with `claudy channel stop` and `claudy channel start`.
- **Channel bot not responding**: check `~/.claudy/channel/logs/server.log` for errors. Verify bot token in `~/.claudy/secrets.env` and that `allowed_users` includes your chat user ID.
- **Permission prompt not appearing**: ensure Claude CLI is not running with `--dangerously-skip-permissions`. The prompt only triggers when Claude needs explicit approval for tool use.
- **Binary not found after install**: ensure Claudy's bin directory is on `PATH`, then restart your shell.
- **Agent not showing in MCP**: ensure the agent binary is on `PATH` (`which gemini`). Only installed agents appear in `tools/list`.
- **Agent timeout**: increase timeout in `config.json` agents field (default: 120s).
- **MCP not registered**: run `claudy mcp` once manually, or check `~/.claude/settings.json` for the `mcpServers.claudy` entry.
- **Agent output truncated**: agent stdout is capped at 10MB. For large outputs, redirect the agent to write to a file instead.
- **Analytics data missing**: run `claudy analytics ingest` to populate from `~/.claude/projects/`. Use `--full` to re-ingest everything.

## Development

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Test analytics backend (uses local DB)
cargo run --example test_dashboard --features analytics-ui

# Launch analytics dashboard (requires analytics-ui feature)
cargo run --features analytics-ui -- analytics dashboard
```

## Contributing

Contributions are welcome! Here is how to get started:

1. Fork the repository and create a feature branch.
2. Make your changes with tests where appropriate.
3. Run `cargo test && cargo clippy -- -D warnings` before submitting.
4. Open a Pull Request at https://github.com/epicsagas/claudy.

Bug reports and feature requests are welcome via [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Acknowledgements

This project was inspired by [Clother](https://github.com/jolehuit/clother), a Go-based multi-provider launcher for Claude CLI. Claudy is an independent Rust implementation, redesigned from the ground up with RAII-based session guards, signal forwarding, launcher symlinks, and deep ecosystem integrations including a **full-featured Channel Bridge** (Telegram/Slack/Discord), the **Agent MCP Bridge** for cross-agent delegation, and a **high-performance Analytics Dashboard** built with Tauri 2. These additions reflect Claudy's transition from a simple launcher to a comprehensive operational toolkit for Claude CLI users.

## License

[Apache-2.0](LICENSE)
