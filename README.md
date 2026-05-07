<h1 align="center">claudy</h1>

<p align="center"><b>One command. Any provider. Full control over Claude CLI.</b></p>

<p align="center">
Stop juggling environment variables and config files.<br/>
Claudy lets you switch between Anthropic, Z.AI, OpenRouter, Ollama, and custom endpoints with a single command — keeping credentials, config modes, and Claude frameworks cleanly isolated per profile.
</p>

<p align="center">
<b>Multi-provider · Config isolation · Channel bridge · Local agent bridge · Usage analytics</b>
</p>

---

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

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
    <a href="https://github.com/epicsagas/claudy/actions/workflows/ci.yml"><img src="https://github.com/epicsagas/claudy/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/features-2048.png">
  <img alt="Why Claudy" src="docs/assets/features-2048.png" width="100%">
</picture>

## Why Claudy

| | Feature | Why it matters |
|--|---------|----------------|
| 🔄 | Multi-provider launch | Switch across Anthropic, Z.AI, OpenRouter, Ollama, and custom endpoints in one command |
| 📦 | Config modes | Isolate `CLAUDE.md`, settings, skills, and agents per mode — no cross-contamination |
| 🔗 | Agent MCP bridge | Delegate tasks from Claude Code to Gemini, Codex, Aider, and 20+ other agents |
| 💬 | Channel bridge | Run Telegram, Slack, and Discord bots with interactive permission prompts |
| 📊 | Usage analytics | Track token usage, costs, and tool patterns with a local Tauri dashboard |
| 🔐 | Safe process control | SIGINT/SIGTERM forwarding, atomic config writes, 0600 credential storage |
| 🛠️ | Operational UX | Install, update, uninstall, doctor, ping — everything from one binary |

## Supported Providers

> Claudy was inspired by [Clother](https://github.com/jolehuit/clother), a Go-based multi-provider launcher for Claude CLI. Z.AI has been the most thoroughly tested provider. If you run into any issues with other providers, please [open an issue](https://github.com/epicsagas/claudy/issues).

| Provider | Status | Notes |
|---|---|---|
| Built-in (Anthropic) | ✅ Tested | Default |
| Z.AI | ✅ Tested | |
| OpenRouter alias | ⚠️ Experimental | Not fully tested — report issues on GitHub |
| Ollama | ⚠️ Experimental | Not fully tested — report issues on GitHub |
| Custom endpoint | ⚠️ Experimental | Not fully tested — report issues on GitHub |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/demo.gif">
  <img alt="demo" src="docs/assets/demo.gif" width="100%">
</picture>

## Quick Start

**1. Install** (pick one)

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh

# macOS (Homebrew)
brew tap epicsagas/tap && brew install claudy

# Windows (PowerShell)
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex

# Cargo (all platforms)
cargo binstall claudy
```

**2. Configure**

```bash
claudy install                        # initialize dirs, config, secrets
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. Launch**

```bash
claudy                                # default provider
claudy zai                            # Z.AI provider
claudy openrouter sonnet              # OpenRouter alias
```

<details>
<summary>Configuration reference</summary>

<details>
<summary>Provider credentials</summary>

| Variable | Provider |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (native) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI China |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax China |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `ALIBABA_API_KEY` | Alibaba Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (all aliases) |

Custom providers use the `api_key_env` variable defined in their `custom_providers` entry.

</details>

<details>
<summary>config.yaml schema</summary>

All configuration lives in `~/.claudy/config.yaml`. Only add the sections you need — defaults are used for anything omitted.

```yaml
# Provider overrides — override default model and model tiers per provider
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"                # → ANTHROPIC_DEFAULT_HAIKU_MODEL
      sonnet: "glm-5.1"               # → ANTHROPIC_DEFAULT_SONNET_MODEL
      opus: "glm-5"                   # → ANTHROPIC_DEFAULT_OPUS_MODEL

# OpenRouter aliases — invoke as: claudy or <alias>
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

# Custom Anthropic-compatible providers — invoke as: claudy <slug>
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

# Compaction policy
compaction:
  auto_compact: true                   # default: true
  threshold: 0.8                       # 0.0–1.0, default: 0.8

# Per-model context window overrides
model_settings:
  deepseek-chat:
    max_context_tokens: 64000

# Channel bridge — non-interactive alternative to `claudy channel add`
channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0           # 0 = unlimited
  stream_timeout_secs: 1800

# Agent overrides
agents:
  aider:
    binary: "aider"
    args: ["--message", "{prompt}"]
    timeout: 300
```

</details>

</details>

---

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

Modes are also a natural fit for **dedicated Claude frameworks and toolkits** that ship their own `CLAUDE.md`, skills, agents, or settings — such as [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code), or any custom harness. Instead of polluting your default config, isolate each framework in its own mode:

```bash
# Create a dedicated mode for the framework
claudy mode create gstack

# Copy or symlink the framework's config into the mode directory
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Launch Claude with that framework active
claudy <profile> gstack
```

Each mode directory is a self-contained `CLAUDE_CONFIG_DIR`, so frameworks never conflict with each other or with your default setup.

<details>
<summary>Command Reference</summary>

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
claudy channel serve [--profile <profile>] [--listen <host:port>]
claudy channel start [--profile <profile>] [--listen <host:port>]
claudy channel stop
claudy channel restart [--profile <profile>] [--listen <host:port>]
claudy channel status
claudy channel add <telegram|slack|discord>
claudy channel remove <telegram|slack|discord>
claudy channel enable
claudy channel disable
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

Store channel credentials in `~/.claudy/secrets.env` (see [Provider credentials](#provider-credentials-secretsenv) for full format):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

<details>
<summary>Agent MCP Bridge</summary>

### Agent MCP bridge

Run `claudy mcp` to start a stdio-based MCP server that lets Claude Code delegate tasks to other locally installed AI coding agents.

```bash
claudy mcp run        # Start the MCP server (called by Claude Code)
claudy mcp install    # Register claudy as an MCP server in Claude Code settings
claudy mcp uninstall  # Remove claudy from Claude Code MCP settings
```

`claudy mcp install` automatically registers itself in `~/.claude/settings.json`. When you create a mode with `claudy mode create <name>`, it also registers in the mode's settings file. No manual configuration needed.

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
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
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

Add agents in `~/.claudy/config.yaml` under the `agents` key (see [Configuration](#configyaml-schema) for full schema):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

Same key as a built-in agent overrides its defaults. `{prompt}` in `args` is replaced with the actual task.

<details>
<summary>Usage Analytics</summary>

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
claudy analytics sync-pricing      # Sync model pricing from models.dev and Anthropic pricing page
claudy analytics recalculate       # Recalculate all costs using the latest pricing data
claudy analytics insights          # Generate compact JSON insights summary (default: 7 days)
claudy analytics insights --days 14  # Analyze last 14 days
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # Specific date range
claudy analytics insights --project my-project  # Filter by project
```

### Inside Claude Code: `/analytics-insights`

The fastest way to analyze your usage is directly inside Claude Code. The `analytics-insights` skill is automatically available — just ask naturally:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude runs `claudy analytics insights`, analyzes the JSON, and returns a structured report with:

- **Cost trends** — daily/weekly spend with spike detection
- **Model distribution** — which models you use and what they cost per session
- **Tool patterns** — most-used tools, error rates, efficiency observations
- **Cache performance** — hit ratio and estimated savings
- **Actionable recommendations** — specific suggestions like "route simple tasks to turbo" with estimated dollar savings

Example output (see [`docs/examples/analytics-insights-sample.json`](docs/examples/analytics-insights-sample.json) for raw data):

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

No manual commands, no context switching. Ask Claude about your usage and get answers instantly.

### What analytics tracks

- **Tokens**: Detailed trends of input, output, and cache tokens over the last 30 days, grouped by model and date.
- **Tools**: Distribution analysis showing which tools Claude uses most frequently, including call counts, error rates, and average execution time.
- **Cost**: Real-time estimation of usage costs based on actual token pricing, including daily/weekly/monthly forecasts and trend detection (increasing/stable/decreasing).
- **Tips (Recommendations)**: Data-driven optimization advice, such as detecting high-cost sessions, suggesting Haiku for simple tasks, and identifying long conversations that could benefit from context summarization.
- **Projects**: Automatically maps cryptic session UUIDs to human-readable project folder names for better context.

Data is stored in a local SQLite database under `~/.claudy/analytics/`. The dashboard runs as a high-performance local Tauri 2 + Svelte app. Use the **[Sync]** button in the dashboard to instantly refresh data from your Claude CLI history.

### Analytics Dashboard
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/analytics-dashboard.png">
  <img alt="Analytics Dashboard" src="docs/assets/analytics-dashboard.png" width="100%">
</picture>

</details>

</details>

</details>

## Files and Directory Layout

By default, Claudy stores data under:

```text
~/.claudy/
```

Important files/directories:

- `config.yaml`: provider + channel + agent configuration.
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

### Run a dedicated Claude framework in its own mode

Frameworks like gstack, superpowers, or ecc ship their own `CLAUDE.md`, skills, and agents. Keep them isolated:

```bash
# One-time setup: create the mode and seed it with the framework config
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Daily use: launch Claude with the framework active
claudy <profile> gstack
```

Switch between frameworks without touching your default config:

```bash
claudy <profile> gstack      # gstack framework active
claudy <profile> superpowers # superpowers framework active
claudy <profile>             # your default config, unchanged
```

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
- **Binary not found after install**: see the PATH note in the [Verify](#verify) section.
- **Agent not showing in MCP**: ensure the agent binary is on `PATH` (`which gemini`). Only installed agents appear in `tools/list`.
- **Agent timeout**: increase timeout in `config.yaml` agents field (default: 120s).
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
