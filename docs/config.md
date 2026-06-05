# config.yaml Reference

All configuration lives in `~/.claudy/config.yaml`. Every section is optional — omit what you don't need and the defaults apply.

- [Top-level structure](#top-level-structure)
- [provider\_overrides](#provider_overrides)
- [openrouter\_aliases](#openrouter_aliases)
- [custom\_providers](#custom_providers)
- [compaction](#compaction)
- [model\_settings](#model_settings)
- [channel](#channel)
- [agents](#agents)
- [Full example](#full-example)

---

## Top-level structure

| Field | Type | Default | Description |
|---|---|---|---|
| `version` | `int` | `1` | Schema version. Reserved for future migrations. |
| `provider_overrides` | `map<string, ModelPreset>` | `{}` | Override default model/tiers per built-in provider. |
| `openrouter_aliases` | `map<string, string>` | `{}` | Short names → OpenRouter model IDs. |
| `custom_providers` | `map<string, UserEndpoint>` | `{}` | Anthropic-compatible third-party endpoints. |
| `compaction` | `ContextWindowPolicy` | see below | Auto-compact behavior. |
| `model_settings` | `map<string, PerModelOverrides>` | `{}` | Per-model context window overrides. |
| `channel` | `BridgeSettings` | see below | Channel bridge (Telegram/Slack/Discord). |
| `agents` | `map<string, AgentConfig>` | `{}` | Agent overrides / custom agents. |

---

## provider_overrides

Override the default model and model-tier mappings for any built-in provider (e.g. `zai`, `anthropic`, `ollama`).

| Field | Type | Default | Description |
|---|---|---|---|
| `model` | `string` | `""` | Default model to use for this provider. Accepts a model ID string or a 1-based index into the provider's `model_choices`. |
| `model_tiers` | `map<string, string>` | `{}` | Map tier name → model ID. Recognized tiers: `opus`, `sonnet`, `haiku`, `small`. Sets `ANTHROPIC_DEFAULT_<TIER>_MODEL`. |

```yaml
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"
      sonnet: "glm-5.1"
      opus: "glm-5"
```

---

## openrouter_aliases

Map short names to OpenRouter model IDs. Invoke with `claudy or <alias>`.

```yaml
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"
  gemini: "google/gemini-2.5-pro"
```

---

## custom_providers

Register Anthropic-compatible third-party endpoints. Invoke with `claudy <slug>`.

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | `string` | yes | Internal identifier (same as the map key). |
| `display_name` | `string` | yes | Human-readable label shown in `claudy ls`. |
| `base_url` | `string` | yes | Endpoint base URL (must be Anthropic-API-compatible). |
| `api_key_env` | `string` | yes | Environment variable that holds the API key (e.g. `MY_LLM_API_KEY`). |
| `default_model` | `string` | no | Default model ID for this provider. |

```yaml
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.example.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"
```

---

## compaction

Control when claudy triggers Claude's context-window compaction.

| Field | Type | Default | Description |
|---|---|---|---|
| `auto_compact` | `bool` | `true` | Enable automatic compaction when the context fills. |
| `threshold` | `float` | `0.8` | Fraction of the context window (0.0–1.0) at which compaction is triggered. |

```yaml
compaction:
  auto_compact: true
  threshold: 0.85
```

---

## model_settings

Per-model context window overrides. The key is any model ID string.

| Field | Type | Description |
|---|---|---|
| `max_context_tokens` | `uint` | Hard cap on context tokens for this model. |
| `compaction_threshold` | `float` | Per-model compaction threshold (0.0–1.0), overrides `compaction.threshold`. |

```yaml
model_settings:
  deepseek-chat:
    max_context_tokens: 64000
  claude-opus-4-5:
    compaction_threshold: 0.9
```

---

## channel

Configure the channel bridge for Telegram, Slack, and Discord.  
Non-interactive alternative to `claudy channel add`.

### Core

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled_platforms` | `string[]` | `[]` | Platforms to activate: `telegram`, `slack`, `discord`. |
| `listen_addr` | `string` | `"127.0.0.1:3456"` | Address:port the bridge HTTP server binds to. |
| `stream_timeout_secs` | `uint` | `1800` | Max seconds to wait for a Claude response stream (30 min). |
| `max_concurrent_sessions` | `uint` | `0` | Max parallel Claude sessions. `0` = unlimited. |

### Provider routing

| Field | Type | Description |
|---|---|---|
| `default_profile` | `string` | Provider profile used when no platform-level override is set. |
| `platform_profiles` | `map<string, string>` | Per-platform profile. Key = platform name (`telegram`/`slack`/`discord`). |
| `channel_profiles` | `map<string, string>` | Per-channel profile. Key = `"platform:channel_id"` or `"platform:guild_id:channel_id"` (Discord). |

Lookup order: `channel_profiles["platform:guild_id:channel_id"]` → `channel_profiles["platform:guild_id"]` → `channel_profiles["platform:channel_id"]` → `platform_profiles["platform"]` → `default_profile`.

### Mode routing

| Field | Type | Description |
|---|---|---|
| `default_mode` | `string` | Mode name applied to all platforms unless overridden. |
| `platform_modes` | `map<string, string>` | Per-platform mode. Key = platform name. |
| `channel_modes` | `map<string, string>` | Per-channel mode. Key format same as `channel_profiles`. |

### Project routing

| Field | Type | Description |
|---|---|---|
| `default_project` | `string` | Absolute path to the default working directory. |
| `channel_projects` | `map<string, string>` | Per-channel project directory. Key format same as `channel_profiles`. |

### Access control

| Field | Type | Description |
|---|---|---|
| `allowed_users` | `string[]` | User IDs / usernames allowed across all platforms. Empty = allow all. |
| `platform_allowed_users` | `map<string, string[]>` | Per-platform override of `allowed_users`. Key = platform name. |

```yaml
channel:
  enabled_platforms: ["telegram", "discord"]
  listen_addr: "127.0.0.1:3456"
  stream_timeout_secs: 1800
  max_concurrent_sessions: 4

  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
    discord: "anthropic"
  channel_profiles:
    "discord:guild123:channel456": "openrouter"

  default_mode: "default"
  platform_modes:
    telegram: "focus"

  default_project: "/home/user/projects/main"
  channel_projects:
    "telegram:987654321": "/home/user/projects/side"

  allowed_users: ["user_id_1", "user_id_2"]
  platform_allowed_users:
    discord: ["discord_user_id_3"]
```

---

## agents

Override built-in agent defaults or register custom agents.  
Built-in agent names: `codex`, `copilot`, `agent`, `agy`, `opencode`, `cline`, `goose`, `amp`, `droid`, `kiro`, `junie`, `kimi`, `vibe`, `qwen-code`, `crush`, `groq-code`, `plandex`, `kilo`, `openhands`.

All fields are optional. For a built-in agent, only the fields present override the defaults.  
For a **custom agent** (any key not in the built-in list), `binary` is required.

| Field | Type | Default | Description |
|---|---|---|---|
| `binary` | `string` | built-in default | Executable name or absolute path. Required for custom agents. |
| `args` | `string[]` | built-in default | Argument list. Use `{prompt}` as a placeholder for the task string. |
| `description` | `string` | built-in default | Human-readable description shown in `claudy mcp list-agents`. |
| `timeout` | `uint` | built-in default | Execution timeout in seconds. Can also be set globally via `CLAUDY_AGENT_TIMEOUT`. |

Priority for timeout: `agents.<name>.timeout` > `CLAUDY_AGENT_TIMEOUT` env var > built-in default.

```yaml
agents:
  # Override timeout only — binary and args stay at built-in defaults
  codex:
    timeout: 7200

  # Full override of a built-in agent
  aider:
    binary: "aider"
    args: ["--message", "{prompt}", "--yes-always"]
    timeout: 600

  # Register a custom agent (binary is required)
  my-agent:
    binary: "/usr/local/bin/my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom coding agent"
    timeout: 300
```

---

## Full example

```yaml
version: 1

provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"
      sonnet: "glm-5.1"
      opus: "glm-5"

openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.example.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

compaction:
  auto_compact: true
  threshold: 0.85

model_settings:
  deepseek-chat:
    max_context_tokens: 64000

channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0
  stream_timeout_secs: 1800

agents:
  codex:
    timeout: 7200
  my-agent:
    binary: "/usr/local/bin/my-agent"
    args: ["--prompt", "{prompt}"]
    description: "My custom coding agent"
    timeout: 300
```
