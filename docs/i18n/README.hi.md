[← English](../../README.md)

<p align="center">
  <a href="README_ko.md">🇰🇷 한국어</a> •
  <a href="README_zh.md">🇨🇳 中文</a> •
  <a href="README_ja.md">🇯🇵 日本語</a> •
  <a href="README_de.md">🇩🇪 Deutsch</a> •
  <a href="README_fr.md">🇫🇷 Français</a> •
  <a href="README_es.md">🇪🇸 Español</a> •
  <a href="README_hi.md">🇮🇳 हिन्दी</a> •
  <a href="README_pt.md">🇧🇷 Português</a> •
  <a href="README_id.md">🇮🇩 Bahasa</a> •
  <a href="README_ar.md">🇸🇦 العربية</a>
</p>

<h1 align="center">claudy</h1>

<p align="center"><b>Claude CLI के लिए आधुनिक मल्टी-provider लॉन्चर।</b></p>

---

<p align="center">
Claudy आपको एक सुसंगत कमांड इंटरफेस के साथ कई providers के विरुद्ध Claude चलाने में मदद करता है, जबकि provider credentials और Claude config overlays को एक ही होम डायरेक्टरी के अंतर्गत व्यवस्थित रखता है।
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../../assets/features-2048.png" alt="Why Claudy" width="100%" />

## Claudy क्यों?

- **मल्टी-provider लॉन्च**: built-in, Z.AI, OpenRouter alias, Ollama और कस्टम Anthropic-compatible endpoints के बीच स्विच करें।
- **Config modes**: प्रत्येक mode के लिए Claude configuration (`CLAUDE.md`, `settings.json`, skills/plugins/agents) को अलग रखें।
- **Provider Profile रिज़ॉल्यूशन**: built-in providers, कस्टम providers और OpenRouter aliases को एकीकृत करें।
- **सुरक्षित process व्यवहार**: SIGINT/SIGTERM को चाइल्ड Claude process को फॉरवर्ड करता है।
- **ऑपरेशनल UX**: install/update/uninstall कमांड, status चेक और connectivity टेस्ट।
- **वैकल्पिक Channel bridge**: Telegram, Slack और Discord के लिए इंटरैक्टिव permission prompts के साथ एक लोकल बॉट bridge चलाएं।
- **Agent MCP bridge**: MCP के ज़रिए Claude Code से अन्य लोकल AI agents (Gemini, Codex, Aider, आदि) को टास्क सौंपें।
- **Usage analytics**: `~/.claude/projects/` से session डेटा इनजेस्ट करें, प्रति session/project टोकन उपयोग और लागत ट्रैक करें, recommendations के साथ एक लोकल dashboard देखें।

## Provider Status

> Claudy को [Clother](https://github.com/jolehuit/clother) से प्रेरणा मिली है, जो Claude CLI के लिए एक Go-आधारित मल्टी-provider लॉन्चर है। केवल **Z.AI provider को पूरी तरह से परीक्षण किया गया है**। अन्य सभी वैकल्पिक providers प्रयोगात्मक और अपरीक्षित हैं — इनका उपयोग अपने जोखिम पर करें।

| Provider | Status | Notes |
|---|---|---|
| Built-in (Anthropic) | ✅ परीक्षित | डिफ़ॉल्ट |
| Z.AI | ✅ परीक्षित | पूरी तरह से मान्य |
| OpenRouter alias | ⚠️ प्रयोगात्मक | अपरीक्षित — अपने जोखिम पर उपयोग करें |
| Ollama | ⚠️ प्रयोगात्मक | अपरीक्षित — अपने जोखिम पर उपयोग करें |
| Custom endpoint | ⚠️ प्रयोगात्मक | अपरीक्षित — अपने जोखिम पर उपयोग करें |

## आवश्यकताएं

- macOS या Linux
- सोर्स से build/install के लिए Rust toolchain (`cargo`)
- Claude CLI इंस्टॉल और `PATH` में उपलब्ध

## इंस्टॉलेशन

### crates.io से इंस्टॉल करें

**Pre-built binary (तेज़, कोई compilation नहीं)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**कोई भी platform — सोर्स से build करें**

```
cargo install claudy
```

**MacOS homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### लोकल सोर्स से इंस्टॉल करें

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### सत्यापित करें

```bash
claudy --help
claudy --version
```

## Quick Start

```bash
# 1) उपलब्ध/resolved profiles की सूची देखें
claudy ls

# 2) credentials को इंटरैक्टिव रूप से कॉन्फ़िगर करें
claudy setup

# 3) एक Profile की विवरण जांचें
claudy show <profile>

# 4) एक Profile के साथ Claude चलाएं
claudy <profile> [claude-args...]
```

## मुख्य अवधारणाएं

### Profile

एक लॉन्च टार्गेट जो provider मेटाडेटा + auth strategy (built-in provider, OpenRouter alias, या कस्टम provider) को रिज़ॉल्व करता है।

### Mode

`~/.claudy/modes/<name>/` पर एक नामित Claude config डायरेक्टरी।

जब आप चलाते हैं:

```bash
claudy <profile> <mode> [args...]
```

Claudy सेट करता है:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

ताकि Claude mode-specific config फ़ाइलें पढ़े।

## कमांड संदर्भ

### मुख्य कमांड

- `claudy ls` (alias: `list`): कॉन्फ़िगर/resolved profiles की सूची।
- `claudy setup [provider]` (alias: `config`): इंटरैक्टिव provider सेटअप।
- `claudy show <profile>` (alias: `info`): resolved provider विवरण दिखाएं।
- `claudy ping [profile]` (alias: `test`): provider connectivity टेस्ट करें।
- `claudy doctor` (alias: `status`): version, paths और profile काउंट दिखाएं।
- `claudy sync` (alias: `install`): claudy binary इंस्टॉल/सिंक्रोनाइज़ करें।
- `claudy update`: claudy अपडेट करें।
- `claudy uninstall`: इंस्टॉल की गई फ़ाइलें हटाएं।
- `claudy mode <action> [name]`: Claude config modes प्रबंधित करें।
- `claudy channel <subcommand>`: Channel bridge प्रबंधित करें।
- `claudy mcp`: agent bridge के लिए MCP server के रूप में चलाएं।
- `claudy analytics <subcommand>`: usage analytics dashboard।

### Mode कमांड

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Mode नाम नियम: `[a-z0-9][a-z0-9_-]*` (`mode` आरक्षित है)।

### Channel कमांड (वैकल्पिक bridge)

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

`channel add` आपको बॉट token, allowed users, Profile और Mode मैपिंग के माध्यम से गाइड करता है।

#### समर्थित platforms

| Platform | Ingestion | इंटरैक्टिव बटन | Notes |
|----------|-----------|----------------|-------|
| Telegram | Long-polling + webhook | Inline keyboard | सबसे पूर्ण |
| Slack | Event subscription webhook | Block Kit actions | HMAC-SHA256 सत्यापित |
| Discord | Interaction webhook | Action row components | Ed25519 सत्यापित |

#### Channel बॉट कमांड

चलने के बाद, बॉट चैट में इन कमांड का जवाब देता है:

- `/help` — उपलब्ध कमांड दिखाएं
- `/cancel` — वर्तमान टास्क रद्द करें
- `/model` — Claude model बदलें (इंटरैक्टिव बटन)
- `/yolo` — auto-allow permissions टॉगल करें
- `/status` — session status, Profile, Mode, git branch और टोकन उपयोग दिखाएं
- `/sessions` — हाल की Claude sessions की सूची (स्विच बटन के साथ)
- `/projects` — projects की सूची (ब्राउज़ बटन के साथ)
- `/new` — नया session शुरू करें
- `/history` — हाल का session इतिहास दिखाएं

Claude से सीधे बात करने के लिए कोई भी अन्य टेक्स्ट भेजें।

#### Permission prompts

जब Claude किसी टूल का उपयोग करने के लिए अनुमोदन मांगता है (कोई कमांड चलाएं, फ़ाइल संपादित करें, आदि), बॉट आपकी चैट में एक इंटरैक्टिव Allow/Deny prompt भेजता है। बटन दबाने से प्रतिक्रिया Claude को वापस भेजी जाती है और processing स्वचालित रूप से जारी रहती है।

#### Secrets

`~/.claudy/secrets.env` में credentials संग्रहित करें:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP bridge

`claudy mcp` चलाएं ताकि एक stdio-आधारित MCP server शुरू हो जो Claude Code को अन्य लोकल AI coding agents को टास्क सौंपने देता है।

```bash
claudy mcp
```

पहली बार चलाने पर, claudy स्वचालित रूप से `~/.claude/settings.json` में खुद को रजिस्टर करता है। जब आप `claudy mode create <name>` के साथ कोई Mode बनाते हैं, तो यह mode की settings फ़ाइल में भी रजिस्टर हो जाता है। कोई मैन्युअल कॉन्फ़िगरेशन आवश्यक नहीं।

मैन्युअल रूप से रजिस्टर करने के लिए (या प्रोजेक्ट-स्तरीय `.claude/settings.json` में):

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

Claude Code को एक `ask_agent` टूल दिखेगा जो सभी इंस्टॉल किए गए agents को expose करता है।

#### उपयोग उदाहरण

एक बार रजिस्टर होने के बाद, Claude Code इस तरह टास्क सौंप सकता है:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code उपयुक्त agent चुनता है, prompt पास करता है और परिणाम लौटाता है। आप एक working directory भी निर्दिष्ट कर सकते हैं:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### MCP रजिस्ट्रेशन सत्यापित करें

```bash
# जांचें कि claudy रजिस्टर है या नहीं
cat ~/.claude/settings.json | grep -A3 claudy

# MCP server को मैन्युअल रूप से टेस्ट करें
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### समर्थित agents (PATH से auto-detected)

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

#### कस्टम agents

`~/.claudy/config.json` में agents जोड़ें:

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

built-in agent के समान key उसके defaults को override करती है। `args` में `{prompt}` को वास्तविक टास्क से बदला जाता है।

### Analytics कमांड

> **नोट**: analytics फीचर अभी भी निर्माणाधीन है। टोकन काउंट, लागत अनुमान और अन्य मेट्रिक्स पूरी तरह से सटीक नहीं हो सकते। आगामी releases में सुधार की उम्मीद है।

```bash
claudy analytics dashboard         # लोकल analytics dashboard खोलें (Tauri 2)
claudy analytics ingest            # ~/.claude/projects/ से session डेटा इनजेस्ट करें
claudy analytics ingest --full     # सभी फ़ाइलें फिर से इनजेस्ट करें (checkpoints अनदेखा करें)
claudy analytics ingest --project my-project  # विशिष्ट project इनजेस्ट करें
claudy analytics recommend         # CLI में usage recommendations दिखाएं
claudy analytics export            # analytics डेटा export करें (JSON, डिफ़ॉल्ट 30 दिन)
claudy analytics export --format csv --days 7  # पिछले 7 दिनों के लिए CSV के रूप में export करें
```

Analytics ट्रैक करता है:

- **Tokens**: पिछले 30 दिनों में input, output और cache tokens के विस्तृत trends, model और date के अनुसार grouped।
- **Tools**: कौन से tools Claude सबसे अधिक बार उपयोग करता है, call counts, error rates और औसत execution time सहित distribution analysis।
- **Cost**: वास्तविक टोकन मूल्य निर्धारण के आधार पर usage costs का real-time अनुमान, daily/weekly/monthly forecasts और trend detection (increasing/stable/decreasing) सहित।
- **Tips (Recommendations)**: डेटा-संचालित optimization सलाह, जैसे high-cost sessions detect करना, सरल कार्यों के लिए Haiku सुझाना, और लंबी conversations की पहचान करना जो context summarization से लाभ उठा सकती हैं।
- **Projects**: बेहतर context के लिए cryptic session UUIDs को human-readable project फ़ोल्डर नामों से स्वचालित रूप से मैप करता है।

डेटा `~/.claudy/analytics/` के अंतर्गत एक लोकल SQLite डेटाबेस में संग्रहीत है। Dashboard एक high-performance लोकल Tauri 2 + Svelte ऐप के रूप में चलता है। अपने Claude CLI इतिहास से डेटा तुरंत रिफ्रेश करने के लिए dashboard में **[Sync]** बटन का उपयोग करें।

<img src="../../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## फ़ाइलें और डायरेक्टरी लेआउट

डिफ़ॉल्ट रूप से, Claudy डेटा इसके अंतर्गत संग्रहीत करता है:

```text
~/.claudy/
```

महत्वपूर्ण फ़ाइलें/डायरेक्टरी:

- `config.json`: provider + channel + agent कॉन्फ़िगरेशन।
- `secrets.env`: provider/bot credentials।
- `launchers.json`: launcher/symlink manifest।
- `modes/`: Claude config modes।
- `session-patches/`: session patch storage।
- `channel/`: channel runtime state (`pid`, sessions, audit log)।
- `analytics/`: analytics SQLite डेटाबेस और checkpoints।
- `cache/update.json`: update metadata cache।

## Environment Variables

- `CLAUDY_HOME`: Claudy होम डायरेक्टरी override करें (डिफ़ॉल्ट: `~/.claudy`)।
- `CLAUDE_CONFIG_DIR`: Mode के साथ launch करते समय Claudy द्वारा स्वचालित रूप से सेट।

## सामान्य Workflows

### Provider कॉन्फ़िगर और लॉन्च करें

```bash
claudy setup
claudy <profile>
```

### Provider के साथ Mode उपयोग करें

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` `--dangerously-skip-permissions` के लिए claudy का shorthand है।

### MCP के ज़रिए अन्य agents को टास्क सौंपें

```bash
# 1) सुनिश्चित करें कि MCP रजिस्टर है (पहले `claudy mcp` पर स्वचालित होता है)
claudy mcp

# 2) Claude Code में, इसे किसी भी इंस्टॉल किए गए agent को delegate करने के लिए कहें:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### install/configuration state का निदान करें

```bash
claudy doctor
claudy ping
```

## समस्या निवारण

- **`profile not recognized`**: `claudy ls` चलाएं और सूचीबद्ध Profile ID चुनें।
- **`not configured` profile**: credentials जोड़ने के लिए `claudy setup <provider>` चलाएं।
- **Channel status unhealthy**: `claudy channel status` चलाएं, फिर `claudy channel stop` और `claudy channel start` से रिस्टार्ट करें।
- **Channel बॉट respond नहीं कर रहा**: errors के लिए `~/.claudy/channel/logs/server.log` जांचें। `~/.claudy/secrets.env` में बॉट token और `allowed_users` में अपना chat user ID शामिल है यह सत्यापित करें।
- **Permission prompt दिखाई नहीं दे रहा**: सुनिश्चित करें कि Claude CLI `--dangerously-skip-permissions` के साथ नहीं चल रहा। prompt केवल तब trigger होता है जब Claude को tool use के लिए स्पष्ट अनुमोदन की आवश्यकता होती है।
- **install के बाद Binary नहीं मिला**: सुनिश्चित करें कि Claudy की bin डायरेक्टरी `PATH` पर है, फिर अपना shell रिस्टार्ट करें।
- **MCP में Agent नहीं दिख रहा**: सुनिश्चित करें कि agent binary `PATH` पर है (`which gemini`)। केवल इंस्टॉल किए गए agents `tools/list` में दिखाई देते हैं।
- **Agent timeout**: `config.json` agents field में timeout बढ़ाएं (डिफ़ॉल्ट: 120s)।
- **MCP रजिस्टर नहीं**: एक बार मैन्युअल रूप से `claudy mcp` चलाएं, या `~/.claude/settings.json` में `mcpServers.claudy` entry जांचें।
- **Agent output truncated**: agent stdout 10MB पर cap है। बड़े outputs के लिए, agent को फ़ाइल में लिखने के लिए redirect करें।
- **Analytics डेटा missing**: `~/.claude/projects/` से populate करने के लिए `claudy analytics ingest` चलाएं। सब कुछ फिर से इनजेस्ट करने के लिए `--full` उपयोग करें।

## विकास

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# analytics backend टेस्ट करें (लोकल DB उपयोग करता है)
cargo run --example test_dashboard --features analytics-ui

# analytics dashboard लॉन्च करें (analytics-ui feature आवश्यक)
cargo run --features analytics-ui -- analytics dashboard
```

## योगदान

योगदान स्वागत है! शुरू करने का तरीका:

1. repository Fork करें और एक feature branch बनाएं।
2. जहां उचित हो tests के साथ अपने changes करें।
3. submit करने से पहले `cargo test && cargo clippy -- -D warnings` चलाएं।
4. https://github.com/epicsagas/claudy पर Pull Request खोलें।

Bug reports और feature requests [GitHub Issues](https://github.com/epicsagas/claudy/issues) के माध्यम से स्वागत हैं।

## आभार

यह प्रोजेक्ट [Clother](https://github.com/jolehuit/clother) से प्रेरित है, जो Claude CLI के लिए एक Go-आधारित मल्टी-provider लॉन्चर है। Claudy एक स्वतंत्र Rust implementation है जिसे शुरू से नए सिरे से डिज़ाइन किया गया है, जिसमें RAII-based session guards, signal forwarding, launcher symlinks और deep ecosystem integrations शामिल हैं जैसे एक **full-featured Channel Bridge** (Telegram/Slack/Discord), cross-agent delegation के लिए **Agent MCP Bridge**, और Tauri 2 के साथ निर्मित **high-performance Analytics Dashboard**। ये additions Claudy के Claude CLI users के लिए एक सरल launcher से एक व्यापक operational toolkit में transition को दर्शाते हैं।

## लाइसेंस

[Apache-2.0](../../LICENSE)
