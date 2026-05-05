[← English](../../README.md)

<h1 align="center">claudy</h1>

<p align="center"><b>一条命令。任意 Provider。完全掌控 Claude CLI。</b></p>

---

<p align="center">
告别繁琐的环境变量和配置文件管理。<br/>
Claudy 让您只需一条命令即可在 Anthropic、Z.AI、OpenRouter、Ollama 及自定义端点之间自由切换，同时将凭据、配置模式和 Claude 框架按 Profile 整洁隔离。
</p>

<p align="center">
<b>多 Provider · 配置隔离 · Channel 桥接 · 本地 Agent 桥接 · 使用分析</b>
</p>

---

<p align="center"><b>适用于 Claude CLI 的现代多 Provider 启动器。</b></p>

---

<p align="center">
Claudy 帮助您通过统一的命令界面在多个 Provider 上运行 Claude，同时将 Provider 凭据和 Claude 配置覆盖整理在单个主目录下。
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../assets/features-2048.png" alt="Why Claudy" width="100%" />

## 为什么选择 Claudy

- **多 Provider 启动**：在内置、Z.AI、OpenRouter 别名、Ollama 和自定义 Anthropic 兼容端点之间切换。
- **Config Mode**：按 Mode 隔离 Claude 配置（`CLAUDE.md`、`settings.json`、技能/插件/代理）。
- **Provider Profile 解析**：统一内置 Provider、自定义 Provider 和 OpenRouter 别名。
- **安全的进程行为**：将 SIGINT/SIGTERM 转发给子 Claude 进程。
- **操作 UX**：安装/更新/卸载命令、状态检查和连接测试。
- **可选 Channel 桥接**：为 Telegram、Slack 和 Discord 运行带有交互式权限提示的本地机器人桥接。
- **Agent MCP 桥接**：通过 MCP 将任务从 Claude Code 委托给其他本地 AI 代理（Gemini、Codex、Aider 等）。
- **使用分析**：从 `~/.claude/projects/` 摄取会话数据，跟踪每个会话/项目的令牌使用量和成本，查看带有建议的本地仪表板。

## 支持的 Provider

> Claudy 受到 [Clother](https://github.com/jolehuit/clother)（一个基于 Go 的 Claude CLI 多 Provider 启动器）的启发。Z.AI 是经过最充分测试的 Provider。如果您在使用其他 Provider 时遇到问题，请[提交 Issue](https://github.com/epicsagas/claudy/issues)。

| Provider | 状态 | 备注 |
|---|---|---|
| 内置 (Anthropic) | ✅ 已测试 | 默认 |
| Z.AI | ✅ 已测试 | |
| OpenRouter 别名 | ⚠️ 实验性 | 尚未完全测试——如遇问题请在 GitHub 提交 Issue |
| Ollama | ⚠️ 实验性 | 尚未完全测试——如遇问题请在 GitHub 提交 Issue |
| 自定义端点 | ⚠️ 实验性 | 尚未完全测试——如遇问题请在 GitHub 提交 Issue |

## 要求

- macOS 或 Linux
- 从源码构建/安装需要 Rust 工具链（`cargo`）
- Claude CLI 已安装且在 `PATH` 中可用

## 安装

### 从 crates.io 安装

**预构建二进制文件（快速，无需编译）**

```
cargo install cargo-binstall
cargo binstall claudy
```

**任意平台——从源码构建**

```
cargo install claudy
```

**macOS Homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### 从本地源码安装

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### 验证

```bash
claudy --help
claudy --version
```

## 快速开始

<img src="docs/assets/demo.gif" alt="Quick Start" width="100%" />

```bash
# 1) 列出可用/已解析的 Profile
claudy ls

# 2) 交互式配置凭据
claudy setup

# 3) 查看一个 Profile 的详细信息
claudy show <profile>

# 4) 使用 Profile 运行 Claude
claudy <profile> [claude-args...]
```

## 核心概念

### Profile

解析 Provider 元数据 + 认证策略（内置 Provider、OpenRouter 别名或自定义 Provider）的启动目标。

### Mode

位于 `~/.claudy/modes/<name>/` 的命名 Claude 配置目录。

当您运行：

```bash
claudy <profile> <mode> [args...]
```

Claudy 设置：

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

这样 Claude 就会读取特定 Mode 的配置文件。

Mode 同样非常适合运行携带自有 `CLAUDE.md`、技能、Agent 或设置的**专用 Claude 框架与工具包** — 例如 [gstack](https://github.com/garrytan/gstack)、[superpowers](https://github.com/obra/superpowers)、[ecc](https://github.com/affaan-m/everything-claude-code) 或自定义工具链。无需污染默认配置，将每个框架隔离到专属 Mode 中即可:

```bash
# 为框架创建专属 Mode
claudy mode create gstack

# 将框架配置复制或符号链接到 Mode 目录
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 激活该框架并启动 Claude
claudy <profile> gstack
```

每个 Mode 目录都是独立的 `CLAUDE_CONFIG_DIR`，框架之间以及与默认配置之间互不冲突。

## 命令参考

### 主要命令

- `claudy ls`（别名：`list`）：列出已配置/已解析的 Profile。
- `claudy setup [provider]`（别名：`config`）：交互式 Provider 设置。
- `claudy show <profile>`（别名：`info`）：显示已解析的 Provider 详细信息。
- `claudy ping [profile]`（别名：`test`）：测试 Provider 连接。
- `claudy doctor`（别名：`status`）：显示版本、路径和 Profile 数量。
- `claudy sync`（别名：`install`）：安装/同步 claudy 二进制文件。
- `claudy update`：更新 claudy。
- `claudy uninstall`：移除已安装的文件。
- `claudy mode <action> [name]`：管理 Claude 配置 Mode。
- `claudy channel <subcommand>`：管理 Channel 桥接。
- `claudy mcp`：作为代理桥接的 MCP 服务器运行。
- `claudy analytics <subcommand>`：使用分析仪表板。

### Mode 命令

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Mode 名称规则：`[a-z0-9][a-z0-9_-]*`（`mode` 为保留字）。

### Channel 命令（可选桥接）

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

`channel add` 引导您完成机器人令牌、允许用户、Profile 和 Mode 映射的配置。

#### 支持的平台

| 平台 | 接收方式 | 交互式按钮 | 备注 |
|----------|-----------|-------------------|-------|
| Telegram | 长轮询 + Webhook | 内联键盘 | 最完整 |
| Slack | 事件订阅 Webhook | Block Kit 操作 | HMAC-SHA256 验证 |
| Discord | 交互 Webhook | Action row 组件 | Ed25519 验证 |

#### Channel 机器人命令

运行后，机器人在聊天中响应以下命令：

- `/help` — 显示可用命令
- `/cancel` — 取消当前任务
- `/model` — 更改 Claude 模型（交互式按钮）
- `/yolo` — 切换自动允许权限
- `/status` — 显示会话状态、Profile、Mode、git 分支和令牌使用量
- `/sessions` — 列出最近的 Claude 会话（带切换按钮）
- `/projects` — 列出项目（带浏览按钮）
- `/new` — 开始新会话
- `/history` — 显示最近的会话历史

发送其他文本直接与 Claude 对话。

#### 权限提示

当 Claude 请求批准使用工具（运行命令、编辑文件等）时，机器人会向您的聊天发送交互式允许/拒绝提示。点击按钮将响应发送回 Claude，处理会自动继续。

#### 密钥

将凭据存储在 `~/.claudy/secrets.env` 中：

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP 桥接

运行 `claudy mcp` 启动基于 stdio 的 MCP 服务器，让 Claude Code 将任务委托给其他本地安装的 AI 编码代理。

```bash
claudy mcp
```

首次运行时，claudy 会自动注册到 `~/.claude/settings.json`。使用 `claudy mode create <name>` 创建 Mode 时，也会注册到该 Mode 的设置文件中。无需手动配置。

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

#### 使用示例

注册后，Claude Code 可以像这样委托任务：

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code 选择适当的代理，传递提示，并返回结果。您也可以指定工作目录：

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### 验证 MCP 注册

```bash
# 检查 claudy 是否已注册
cat ~/.claude/settings.json | grep -A3 claudy

# 手动测试 MCP 服务器
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### 支持的代理（从 PATH 自动检测）

| 代理 | 二进制文件 | 无头命令 |
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

#### 自定义代理

在 `~/.claudy/config.yaml` 中添加代理：

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

与内置代理相同的键会覆盖其默认值。`args` 中的 `{prompt}` 会被实际任务替换。

### 分析命令

> **注意**：分析功能仍在开发中。令牌计数、成本估算和其他指标可能不完全准确。预计在即将发布的版本中进行改进。

```bash
claudy analytics dashboard         # 打开本地分析仪表板（Tauri 2）
claudy analytics ingest            # 从 ~/.claude/projects/ 摄取会话数据
claudy analytics ingest --full     # 重新摄取所有文件（忽略检查点）
claudy analytics ingest --project my-project  # 摄取特定项目
claudy analytics recommend         # 在 CLI 中显示使用建议
claudy analytics export            # 导出分析数据（JSON，默认 30 天）
claudy analytics export --format csv --days 7  # 以 CSV 格式导出最近 7 天的数据
claudy analytics insights          # 生成 LLM 分析用的紧凑 JSON 摘要（默认 7 天）
claudy analytics insights --days 14  # 分析最近 14 天
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # 指定日期范围
claudy analytics insights --project my-project  # 按项目筛选
```

分析跟踪：

- **令牌**：按模型和日期分组的过去 30 天输入、输出和缓存令牌的详细趋势。
- **工具**：显示 Claude 最常用工具的分布分析，包括调用次数、错误率和平均执行时间。
- **成本**：基于实际令牌定价的实时使用成本估算，包括日/周/月预测和趋势检测（增加/稳定/减少）。
- **提示（建议）**：数据驱动的优化建议，如检测高成本会话、建议对简单任务使用 Haiku，以及识别可从上下文摘要中受益的长对话。
- **洞察（LLM 驱动）**：为 LLM 分析优化的紧凑 JSON 格式使用摘要。将成本趋势、模型分布、工具模式、缓存效率和重要会话合并为单个载荷（约 2-3K token）。可通过 Claude Code 的 `analytics-insights` 技能使用自然语言提问（"使用 패턴 分析"、"analyze my usage patterns"），Claude 将生成个性化推荐。
- **项目**：自动将难以理解的会话 UUID 映射到可读的项目文件夹名称。

数据存储在 `~/.claudy/analytics/` 下的本地 SQLite 数据库中。仪表板作为高性能本地 Tauri 2 + Svelte 应用运行。使用仪表板中的 **[Sync]** 按钮立即从您的 Claude CLI 历史记录中刷新数据。

<img src="../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## 文件和目录布局

默认情况下，Claudy 将数据存储在：

```text
~/.claudy/
```

重要文件/目录：

- `config.yaml`：Provider + Channel + 代理配置。
- `secrets.env`：Provider/机器人凭据。
- `launchers.json`：启动器/符号链接清单。
- `modes/`：Claude 配置 Mode。
- `session-patches/`：会话补丁存储。
- `channel/`：Channel 运行时状态（`pid`、会话、审计日志）。
- `analytics/`：分析 SQLite 数据库和检查点。
- `cache/update.json`：更新元数据缓存。

## 环境变量

- `CLAUDY_HOME`：覆盖 Claudy 主目录（默认：`~/.claudy`）。
- `CLAUDE_CONFIG_DIR`：使用 Mode 启动时由 Claudy 自动设置。

## 常见工作流程

### 配置并启动 Provider

```bash
claudy setup
claudy <profile>
```

### 与 Provider 一起使用 Mode

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` 是 claudy 中 `--dangerously-skip-permissions` 的简写。

### 在专属 Mode 中运行 Claude 框架

gstack、superpowers、ecc 等框架自带 `CLAUDE.md`、技能和 Agent，将它们隔离运行：

```bash
# 一次性设置：创建 Mode 并载入框架配置
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 日常使用：激活该框架并启动 Claude
claudy <profile> gstack
```

在框架之间切换，无需修改默认配置：

```bash
claudy <profile> gstack      # 启用 gstack 框架
claudy <profile> superpowers # 启用 superpowers 框架
claudy <profile>             # 默认配置，保持不变
```

### 通过 MCP 将任务委托给其他代理

```bash
# 1) 确保 MCP 已注册（第一次运行 `claudy mcp` 时自动完成）
claudy mcp

# 2) 在 Claude Code 中，要求委托给任何已安装的代理：
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### 诊断安装/配置状态

```bash
claudy doctor
claudy ping
```

## 故障排除

- **`profile not recognized`**：运行 `claudy ls` 并选择列出的 Profile ID。
- **`not configured` Profile**：运行 `claudy setup <provider>` 添加凭据。
- **Channel 状态不健康**：运行 `claudy channel status`，然后使用 `claudy channel stop` 和 `claudy channel start` 重启。
- **Channel 机器人无响应**：检查 `~/.claudy/channel/logs/server.log` 中的错误。验证 `~/.claudy/secrets.env` 中的机器人令牌以及 `allowed_users` 是否包含您的聊天用户 ID。
- **权限提示未出现**：确保 Claude CLI 未使用 `--dangerously-skip-permissions` 运行。提示仅在 Claude 需要工具使用的明确批准时触发。
- **安装后找不到二进制文件**：确保 Claudy 的 bin 目录在 `PATH` 中，然后重启 shell。
- **MCP 中未显示代理**：确保代理二进制文件在 `PATH` 中（`which gemini`）。只有已安装的代理才会出现在 `tools/list` 中。
- **代理超时**：在 `config.yaml` 的 agents 字段中增加超时时间（默认：120 秒）。
- **MCP 未注册**：手动运行一次 `claudy mcp`，或检查 `~/.claude/settings.json` 中的 `mcpServers.claudy` 条目。
- **代理输出被截断**：代理 stdout 上限为 10MB。对于大输出，请将代理重定向到写入文件。
- **分析数据缺失**：运行 `claudy analytics ingest` 从 `~/.claude/projects/` 填充数据。使用 `--full` 重新摄取所有内容。

## 开发

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# 测试分析后端（使用本地数据库）
cargo run --example test_dashboard --features analytics-ui

# 启动分析仪表板（需要 analytics-ui 功能）
cargo run --features analytics-ui -- analytics dashboard
```

## 贡献

欢迎贡献！以下是入门方法：

1. Fork 仓库并创建功能分支。
2. 在适当的情况下进行带测试的更改。
3. 提交前运行 `cargo test && cargo clippy -- -D warnings`。
4. 在 https://github.com/epicsagas/claudy 开启 Pull Request。

欢迎通过 [GitHub Issues](https://github.com/epicsagas/claudy/issues) 提交错误报告和功能请求。

## 致谢

本项目受到 [Clother](https://github.com/jolehuit/clother)（一个基于 Go 的 Claude CLI 多 Provider 启动器）的启发。Claudy 是从零开始重新设计的独立 Rust 实现，引入了基于 RAII 的会话守卫、信号转发、启动器符号链接，以及包括**全功能 Channel 桥接**（Telegram/Slack/Discord）、用于跨代理委托的 **Agent MCP 桥接**，以及使用 Tauri 2 构建的**高性能分析仪表板**在内的深度生态系统集成。这些新增功能反映了 Claudy 从简单启动器到 Claude CLI 用户综合运营工具包的转变。

## 许可证

[Apache-2.0](../../LICENSE)
