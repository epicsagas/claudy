<h1 align="center">claudy</h1>

<p align="center"><b>一条命令，任意提供商，完全掌控 Claude CLI。</b></p>

<p align="center">
不再为环境变量和配置文件头疼。<br/>
Claudy 让你在 Anthropic、Z.AI、OpenRouter、Ollama 和自定义端点之间一键切换 —— 每个配置文件的凭证、配置模式和 Claude 框架都保持干净隔离。
</p>

<p align="center">
<b>多提供商 · 配置隔离 · 频道桥接 · 本地代理桥接 · 使用分析</b>
</p>

---

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> •
  <a href="README.ko.md">🇰🇷 한국어</a> •
  <a href="README.zh-Hans.md">🇨🇳 中文</a> •
  <a href="README.ja.md">🇯🇵 日本語</a> •
  <a href="README.de.md">🇩🇪 Deutsch</a> •
  <a href="README.fr.md">🇫🇷 Français</a> •
  <a href="README.es.md">🇪🇸 Español</a> •
  <a href="README.hi.md">🇮🇳 हिन्दी</a> •
  <a href="README.pt-BR.md">🇧🇷 Português</a> •
  <a href="README.id.md">🇮🇩 Bahasa</a> •
  <a href="README.ar.md">🇸🇦 العربية</a>
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
    <a href="https://github.com/epicsagas/claudy/actions/workflows/ci.yml"><img src="https://github.com/epicsagas/claudy/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features-2048.png">
  <img alt="为什么选择 Claudy" src="../assets/features-2048.png" width="100%">
</picture>

## 为什么选择 Claudy

| | 功能 | 为什么重要 |
|--|------|-----------|
| 🔄 | 多提供商启动 | 一条命令在 Anthropic、Z.AI、OpenRouter、Ollama 和自定义端点之间切换 |
| 📦 | 配置模式 | 每个模式隔离 `CLAUDE.md`、设置、技能和代理 —— 无交叉污染 |
| 🔗 | 代理 MCP 桥接 | 从 Claude Code 将任务委派给 Gemini、Codex、Aider 等 20+ 代理 |
| 💬 | 频道桥接 | 运行 Telegram、Slack 和 Discord 机器人，支持交互式权限提示 |
| 📊 | 使用分析 | 通过本地 Tauri 仪表板追踪 token 用量、成本和工具使用模式 |
| 🔐 | 安全进程控制 | SIGINT/SIGTERM 信号转发、原子配置写入、0600 凭证存储 |
| 🔀 | 跨提供商会话连续性 | 自动修复 Z.AI/GLM 创建的会话，使其可以通过 Anthropic API 无缝续接 |
| 🛠️ | 运维体验 | 安装、更新、卸载、诊断、连通测试 —— 一个二进制文件搞定一切 |

## 支持的提供商

> Claudy 的灵感来自 [Clother](https://github.com/jolehuit/clother)，一个基于 Go 的 Claude CLI 多提供商启动器。Z.AI 是经过最充分测试的提供商。如果在使用其他提供商时遇到问题，请[提交 Issue](https://github.com/epicsagas/claudy/issues)。

| 提供商 | 状态 | 备注 |
|--------|------|------|
| 内置 (Anthropic) | ✅ 已测试 | 默认提供商 |
| Z.AI | ✅ 已测试 | |
| OpenRouter 别名 | ⚠️ 实验性 | 未完全测试 —— 请在 GitHub 上报告问题 |
| Ollama | ⚠️ 实验性 | 未完全测试 —— 请在 GitHub 上报告问题 |
| 自定义端点 | ⚠️ 实验性 | 未完全测试 —— 请在 GitHub 上报告问题 |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="演示" src="../assets/demo.gif" width="100%">
</picture>

## 快速开始

**1. 安装**

macOS / Linux：

```bash
brew install epicsagas/tap/claudy
```

没有 Homebrew？使用安装脚本：

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows：

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

通过 Rust 工具链：

```bash
cargo binstall claudy   # 预编译二进制（快速）
cargo install claudy    # 从源码编译
```

**2. 配置**

```bash
claudy install                        # 初始化目录、配置和密钥
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. 启动**

```bash
claudy                                # 默认提供商
claudy zai                            # Z.AI 提供商
claudy openrouter sonnet              # OpenRouter 别名
```

**4. 更新**

```bash
brew upgrade claudy          # Homebrew
claudy update                # 内置更新器
# 或重新运行安装脚本 / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>提供商凭证</summary>

| 变量 | 提供商 |
|------|--------|
| `ANTHROPIC_API_KEY` | Anthropic (原生) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI 中国 |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax 中国 |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | 小米 MiMo |
| `ALIBABA_API_KEY` | 阿里巴巴 Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (所有别名) |

自定义提供商使用其 `custom_providers` 条目中定义的 `api_key_env` 变量。

</details>

<details>
<summary>config.yaml 模式</summary>

所有配置都存放在 `~/.claudy/config.yaml` 中。只需添加你需要的部分 —— 省略的部分将使用默认值。

> 完整参考: [docs/config.md](../config.md)

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

---

## 核心概念

### 配置文件

一个启动目标，用于解析提供商元数据和认证策略（内置提供商、OpenRouter 别名或自定义提供商）。

### 模式

一个位于 `~/.claudy/modes/<name>/` 的命名 Claude 配置目录。

当你运行：

```bash
claudy <profile> <mode> [args...]
```

Claudy 会设置：

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

这样 Claude 就会读取特定于该模式的配置文件。

模式也天然适合**专用 Claude 框架和工具包**，这些框架自带 `CLAUDE.md`、技能、代理或设置 —— 例如 [gstack](https://github.com/garrytan/gstack)、[superpowers](https://github.com/obra/superpowers)、[ecc](https://github.com/affaan-m/everything-claude-code) 或任何自定义工具链。无需污染你的默认配置，每个框架都可以隔离在自己的模式中：

```bash
# 为框架创建专用模式
claudy mode create gstack

# 将框架的配置复制或链接到模式目录
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 启动 Claude 并激活该框架
claudy <profile> gstack
```

每个模式目录都是一个独立的 `CLAUDE_CONFIG_DIR`，因此框架之间不会相互冲突，也不会影响你的默认设置。

<details>
<summary>命令参考</summary>

## 命令参考

### 主要命令

- `claudy ls`（别名：`list`）：列出已配置/已解析的配置文件。
- `claudy setup [provider]`（别名：`config`）：交互式提供商设置。
- `claudy show <profile>`（别名：`info`）：显示已解析的提供商详情。
- `claudy ping [profile]`（别名：`test`）：测试提供商连通性。
- `claudy doctor`（别名：`status`）：显示版本、路径和配置文件数量。
- `claudy sync`（别名：`install`）：安装/同步 claudy 二进制文件。
- `claudy update`：更新 claudy。
- `claudy uninstall`：移除已安装的文件。
- `claudy mode <action> [name]`：管理 Claude 配置模式。
- `claudy channel <subcommand>`：管理频道桥接。
- `claudy mcp`：作为 MCP 服务器运行，用于代理桥接。
- `claudy analytics <subcommand>`：使用分析仪表板。
- `claudy session sanitize`：修复包含非 Anthropic 提供商写入的无效 thinking 块的会话。

### 模式命令

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

模式名称规则：`[a-z0-9][a-z0-9_-]*`（`mode` 为保留名称）。

### 频道命令（可选桥接）

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

`channel add` 会引导你完成机器人令牌、允许的用户、配置文件和模式映射的设置。

#### 支持的平台

| 平台 | 接入方式 | 交互式按钮 | 备注 |
|------|----------|------------|------|
| Telegram | 长轮询 + Webhook | 内联键盘 | 功能最完整 |
| Slack | 事件订阅 Webhook | Block Kit 操作 | HMAC-SHA256 验证 |
| Discord | 交互 Webhook | 操作行组件 | Ed25519 验证 |

#### 频道机器人命令

机器人运行后，在聊天中响应以下命令：

- `/help` — 显示可用命令
- `/cancel` — 取消当前任务
- `/model` — 更改 Claude 模型（交互式按钮）
- `/yolo` — 切换自动允许权限
- `/status` — 显示会话状态、配置文件、模式、Git 分支和 token 用量
- `/sessions` — 列出最近的 Claude 会话（带切换按钮）
- `/projects` — 列出项目（带浏览按钮）
- `/new` — 开始新会话
- `/history` — 显示最近的会话历史

发送任何其他文本即可直接与 Claude 对话。

#### 权限提示

当 Claude 请求批准使用某个工具（运行命令、编辑文件等）时，
机器人会向你的聊天发送一个交互式的允许/拒绝提示。点击按钮
即可将响应发回 Claude，处理会自动继续。

#### 密钥

将频道凭证存储在 `~/.claudy/secrets.env` 中（完整格式参见[提供商凭证](#提供商凭证secretssenv)）：

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## 代理 MCP 桥接

运行 `claudy mcp` 启动一个基于 stdio 的 MCP 服务器，让 Claude Code 可以将任务委派给其他本地安装的 AI 编程代理。

```bash
claudy mcp run        # 启动 MCP 服务器（由 Claude Code 调用）
claudy mcp install    # 在 Claude Code 设置中注册 claudy 为 MCP 服务器
claudy mcp uninstall  # 从 Claude Code MCP 设置中移除 claudy
```

`claudy mcp install` 会自动在 `~/.claude/settings.json` 中注册。当你使用 `claudy mode create <name>` 创建模式时，它也会在模式的设置文件中注册。无需手动配置。

手动注册（或在项目级 `.claude/settings.json` 中）：

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

Claude Code 将看到一个 `ask_agent` 工具，该工具暴露所有已安装的代理。

### 使用示例

注册完成后，Claude Code 可以像这样委派任务：

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code 会选择合适的代理，传递提示，并返回结果。你也可以指定工作目录：

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### 验证 MCP 注册

```bash
# 检查 claudy 是否已注册
cat ~/.claude/settings.json | grep -A3 claudy

# 手动测试 MCP 服务器
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### 支持的代理（从 PATH 自动检测）

| Agent | Binary | Headless command |
|-------|--------|-----------------|
| Antigravity | `gemini` | `gemini -p "..." --output-format text` |
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

### 自定义代理

在 `~/.claudy/config.yaml` 的 `agents` 键下添加代理（完整模式参见[配置](#configyaml-schema)）：

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

如果与内置代理使用相同的键，则会覆盖其默认值。`args` 中的 `{prompt}` 会被替换为实际的任务内容。

## 使用分析

> **注意**：分析功能仍在开发中。Token 计数、成本估算和其他指标可能不完全准确。后续版本将持续改进。

```bash
claudy analytics dashboard         # 打开本地分析仪表板 (Tauri 2)
claudy analytics ingest            # 从 ~/.claude/projects/ 导入会话数据
claudy analytics ingest --full     # 重新导入所有文件（忽略检查点）
claudy analytics ingest --project my-project  # 导入特定项目
claudy analytics recommend         # 在 CLI 中显示使用建议
claudy analytics export            # 导出分析数据 (JSON，默认 30 天)
claudy analytics export --format csv --days 7  # 导出最近 7 天的 CSV 数据
claudy analytics sync-pricing      # 从 models.dev 和 Anthropic 定价页面同步模型定价
claudy analytics recalculate       # 使用最新定价数据重新计算所有成本
claudy analytics insights          # 生成紧凑的 JSON 分析摘要（默认：7 天）
claudy analytics insights --days 14  # 分析最近 14 天
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # 指定日期范围
claudy analytics insights --project my-project  # 按项目筛选
```

### 在 Claude Code 中：`/analytics-insights`

分析使用情况的最快方式是直接在 Claude Code 中。`analytics-insights` 技能自动可用 —— 只需自然地提问：

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude 会运行 `claudy analytics insights`，分析 JSON 数据，并返回包含以下内容的结构化报告：

- **成本趋势** — 每日/每周支出及尖峰检测
- **模型分布** — 你使用的模型及其每次会话的成本
- **工具使用模式** — 最常用的工具、错误率和效率观察
- **缓存性能** — 命中率和预估节省
- **可操作建议** — 具体建议如"将简单任务路由到 turbo"及预估节省金额

示例输出（原始数据参见 [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json)）：

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

无需手动命令，无需切换上下文。直接向 Claude 询问你的使用情况，即刻获得答案。

### 分析跟踪内容

- **Token**：过去 30 天内 input、output 和 cache token 的详细趋势，按模型和日期分组。
- **工具**：分布分析，显示 Claude 最常使用的工具，包括调用次数、错误率和平均执行时间。
- **成本**：基于实际 token 定价的实时成本估算，包括每日/每周/每月预测和趋势检测（上升/稳定/下降）。
- **提示（建议）**：数据驱动的优化建议，如检测高成本会话、建议使用 Haiku 处理简单任务，以及识别可通过上下文摘要优化的长对话。
- **项目**：自动将晦涩的会话 UUID 映射为可读的项目文件夹名称，提供更好的上下文。

数据存储在 `~/.claudy/analytics/` 下的本地 SQLite 数据库中。仪表板是一个高性能的本地 Tauri 2 + Svelte 应用。使用仪表板中的 **[Sync]** 按钮可立即从 Claude CLI 历史记录刷新数据。

### 分析仪表板
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="分析仪表板" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## 跨提供商会话连续性

使用 Z.AI / GLM 等非 Anthropic 提供商工作时，会话 JSONL 文件中会记录带有空 signature 的 thinking 块。通过 Anthropic API 恢复该会话时会出现以下错误：

```
API Error: 400 Invalid `signature` in `thinking` block
```

Claudy 通过两种方式处理此问题：

**自动处理（频道桥接）：** 频道服务器恢复会话时，会自动将带有空 signature 的 thinking 块转换为普通文本块。无需任何操作。

**手动处理（CLI）：** 在使用 `claude --resume` 直接恢复前，运行 `claudy session sanitize` 修复会话：

```bash
# 交互式 — 从问题会话列表中选择
claudy session sanitize

# 按项目名称过滤
claudy session sanitize --project book-forge

# 批量处理所有问题会话
claudy session sanitize --all --yes
```

**转换方式：** 带有空 signature 的 thinking 块被重写为纯文本块，推理内容以文本形式保留，会话文件以原子方式更新。具有有效 Anthropic signature 的块不会被修改。

**限制：** 会话连续性取决于对话历史的兼容性。在会话中途切换提供商，即使经过 sanitization 也可能产生细微的上下文变化。

---

## 文件和目录结构

默认情况下，Claudy 将数据存储在：

```text
~/.claudy/
```

重要文件和目录：

- `config.yaml`：提供商 + 频道 + 代理配置。
- `secrets.env`：提供商/机器人凭证。
- `launchers.json`：启动器/符号链接清单。
- `modes/`：Claude 配置模式。
- `session-patches/`：会话补丁存储。
- `channel/`：频道运行时状态（`pid`、会话、审计日志）。
- `analytics/`：分析 SQLite 数据库和检查点。
- `cache/update.json`：更新元数据缓存。

## 环境变量

- `CLAUDY_HOME`：覆盖 Claudy 主目录（默认：`~/.claudy`）。
- `CLAUDE_CONFIG_DIR`：Claudy 在使用模式启动时自动设置。

## 常用工作流

### 配置并启动提供商

```bash
claudy setup
claudy <profile>
```

### 使用模式配合提供商

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` 是 claudy 中 `--dangerously-skip-permissions` 的简写。

### 在独立模式中运行专用 Claude 框架

gstack、superpowers 或 ecc 等框架自带 `CLAUDE.md`、技能和代理。将它们保持隔离：

```bash
# 一次性设置：创建模式并导入框架配置
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 日常使用：启动 Claude 并激活框架
claudy <profile> gstack
```

在框架之间切换，无需触及你的默认配置：

```bash
claudy <profile> gstack      # 激活 gstack 框架
claudy <profile> superpowers # 激活 superpowers 框架
claudy <profile>             # 你的默认配置，保持不变
```

### 通过 MCP 将任务委派给其他代理

```bash
# 1) 确保 MCP 已注册（首次运行 `claudy mcp` 时自动完成）
claudy mcp

# 2) 在 Claude Code 中，请求委派给任何已安装的代理：
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### 诊断安装/配置状态

```bash
claudy doctor
claudy ping
```

## 故障排除

- **`profile not recognized`**：运行 `claudy ls` 并选择列出的配置文件 ID。
- **`not configured` 配置文件**：运行 `claudy setup <provider>` 添加凭证。
- **频道状态不健康**：运行 `claudy channel status`，然后使用 `claudy channel stop` 和 `claudy channel start` 重启。
- **频道机器人无响应**：检查 `~/.claudy/channel/logs/server.log` 中的错误。验证 `~/.claudy/secrets.env` 中的机器人令牌，以及 `allowed_users` 是否包含你的聊天用户 ID。
- **权限提示未出现**：确保 Claude CLI 未使用 `--dangerously-skip-permissions` 运行。该提示仅在 Claude 需要明确批准工具使用时触发。
- **安装后找不到二进制文件**：参见 [Verify](#verify) 部分中的 PATH 说明。
- **代理未在 MCP 中显示**：确保代理二进制文件在 `PATH` 中（`which gemini`）。只有已安装的代理才会出现在 `tools/list` 中。
- **代理超时**：在 `config.yaml` 的 agents 字段中增加超时时间（默认：120 秒）。
- **MCP 未注册**：手动运行一次 `claudy mcp`，或检查 `~/.claude/settings.json` 中的 `mcpServers.claudy` 条目。
- **代理输出被截断**：代理 stdout 上限为 10MB。对于大输出，请将代理重定向到写入文件。
- **分析数据缺失**：运行 `claudy analytics ingest` 从 `~/.claude/projects/` 填充数据。使用 `--full` 重新导入所有内容。
- **恢复会话时出现 `400 Invalid signature in thinking block`**：该会话由 Z.AI 等非 Anthropic 提供商创建。运行 `claudy session sanitize` 转换无效的 thinking 块，然后正常恢复。

## 开发

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# 测试分析后端（使用本地数据库）
cargo run --example test_dashboard --features analytics-ui

# 启动分析仪表板（需要 analytics-ui 特性）
cargo run --features analytics-ui -- analytics dashboard
```

## 贡献

欢迎贡献！以下是入门步骤：

1. Fork 仓库并创建功能分支。
2. 进行修改并适当添加测试。
3. 提交前运行 `cargo test && cargo clippy -- -D warnings`。
4. 在 https://github.com/epicsagas/claudy 提交 Pull Request。

欢迎通过 [GitHub Issues](https://github.com/epicsagas/claudy/issues) 提交 Bug 报告和功能请求。

## 致谢

本项目的灵感来自 [Clother](https://github.com/jolehuit/clother)，一个基于 Go 的 Claude CLI 多提供商启动器。Claudy 是一个独立的 Rust 实现，从零开始重新设计，具有基于 RAII 的会话守卫、信号转发、启动器符号链接和深度生态集成，包括**全功能频道桥接**（Telegram/Slack/Discord）、用于跨代理委派的**代理 MCP 桥接**，以及使用 Tauri 2 构建的**高性能分析仪表板**。这些新增功能标志着 Claudy 从一个简单的启动器转变为 Claude CLI 用户的综合运维工具包。

## 许可证

[Apache-2.0](../../LICENSE)
