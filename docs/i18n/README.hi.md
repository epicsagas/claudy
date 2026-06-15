<h1 align="center">claudy</h1>

<p align="center"><b>एक कमांड। कोई भी प्रदाता। Claude CLI पर पूर्ण नियंत्रण।</b></p>

<p align="center">
पर्यावरण चर और कॉन्फ़िग फ़ाइलों को सँभालना बंद करें।<br/>
Claudy आपको Anthropic, Z.AI, OpenRouter, Ollama, और कस्टम एंडपॉइंट के बीच एक ही कमांड से स्विच करने देता है — क्रेडेंशियल, कॉन्फ़िग मोड, और Claude फ़्रेमवर्क को प्रति प्रोफ़ाइल साफ़ रूप से अलग रखता है।
</p>

<p align="center">
<b>मल्टी-प्रदाता · कॉन्फ़िग विलगीकरण · चैनल ब्रिज · लोकल एजेंट ब्रिज · उपयोग विश्लेषण</b>
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
  <img alt="Claudy सुविधाएँ" src="../assets/features-2048.png" width="100%">
</picture>

## Claudy क्यों?

| | सुविधा | यह क्यों मायने रखती है |
|--|---------|------------------------|
| 🔄 | मल्टी-प्रदाता लॉन्च | Anthropic, Z.AI, OpenRouter, Ollama, और कस्टम एंडपॉइंट के बीच एक ही कमांड में स्विच करें |
| 📦 | कॉन्फ़िग मोड | `CLAUDE.md`, सेटिंग्स, स्किल, और एजेंट को प्रति मोड अलग रखें — कोई क्रॉस-कंटेमिनेशन नहीं |
| 🔗 | एजेंट MCP ब्रिज | Claude Code से agy, Codex, Aider और 20+ अन्य एजेंटों को कार्य सौंपें |
| 💬 | चैनल ब्रिज | इंटरैक्टिव अनुमति प्रॉम्प्ट के साथ Telegram, Slack, और Discord बॉट चलाएँ |
| 📊 | उपयोग विश्लेषण | लोकल Tauri डैशबोर्ड से टोकन उपयोग, लागत, और टूल पैटर्न ट्रैक करें |
| 🔐 | सुरक्षित प्रोसेस नियंत्रण | SIGINT/SIGTERM फ़ॉरवर्डिंग, परमाणु कॉन्फ़िग राइट, 0600 क्रेडेंशियल स्टोरेज |
| 🔀 | क्रॉस-प्रोवाइडर सेशन निरंतरता | Z.AI/GLM सेशन को Anthropic API के साथ बिना रुकावट जारी रखने के लिए स्वचालित रूप से ठीक करें |
| 🛠️ | संचालन UX | इंस्टॉल, अपडेट, अनइंस्टॉल, डॉक्टर, पिंग — सब कुछ एक बाइनरी से |

## समर्थित प्रदाता

> Claudy [Clother](https://github.com/jolehuit/clother) से प्रेरित है, जो Claude CLI के लिए Go-आधारित मल्टी-प्रदाता लॉन्चर है। Z.AI सबसे अधिक परीक्षित प्रदाता रहा है। यदि अन्य प्रदाताओं के साथ कोई समस्या आती है, तो कृपया [एक इश्यू खोलें](https://github.com/epicsagas/claudy/issues)।

| प्रदाता | स्थिति | टिप्पणी |
|---|---|---|
| बिल्ट-इन (Anthropic) | ✅ परीक्षित | डिफ़ॉल्ट |
| Z.AI | ✅ परीक्षित | |
| OpenRouter उपनाम | ⚠️ प्रायोगिक | पूर्ण रूप से परीक्षित नहीं — GitHub पर इश्यू रिपोर्ट करें |
| Ollama | ⚠️ प्रायोगिक | पूर्ण रूप से परीक्षित नहीं — GitHub पर इश्यू रिपोर्ट करें |
| कस्टम एंडपॉइंट | ⚠️ प्रायोगिक | पूर्ण रूप से परीक्षित नहीं — GitHub पर इश्यू रिपोर्ट करें |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="डेमो" src="../assets/demo.gif" width="100%">
</picture>

## त्वरित शुरुआत

**1. इंस्टॉल करें**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

Homebrew नहीं है? इंस्टॉलर स्क्रिप्ट उपयोग करें:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Rust टूलचेन के माध्यम से:

```bash
cargo binstall claudy   # प्री-बिल्ट बाइनरी (तेज़)
cargo install claudy    # सोर्स से बिल्ड
```

**2. कॉन्फ़िगर करें**

```bash
claudy install                        # डायरेक्ट्री, कॉन्फ़िग, सीक्रेट इनिशियलाइज़ करें
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. लॉन्च करें**

```bash
claudy                                # डिफ़ॉल्ट प्रदाता
claudy zai                            # Z.AI प्रदाता
claudy openrouter sonnet              # OpenRouter उपनाम
```

**4. अपडेट करें**

```bash
brew upgrade claudy          # Homebrew
claudy update                # बिल्ट-इन अपडेटर
# या इंस्टॉलर स्क्रिप्ट / cargo binstall claudy@latest पुनः चलाएँ
claudy --version
```

<details>
<summary>प्रदाता क्रेडेंशियल</summary>

| चर | प्रदाता |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (नेटिव) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI चीन |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax चीन |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `ALIBABA_API_KEY` | Alibaba Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (सभी उपनाम) |

कस्टम प्रदाता अपने `custom_providers` प्रविष्टि में परिभाषित `api_key_env` चर का उपयोग करते हैं।

</details>

<details>
<summary>config.yaml स्कीमा</summary>

सारा कॉन्फ़िगरेशन `~/.claudy/config.yaml` में रहता है। केवल आवश्यक सेक्शन जोड़ें — छोड़े गए आइटम के लिए डिफ़ॉल्ट मान उपयोग होते हैं।

> पूर्ण संदर्भ: [docs/config.md](../config.md)

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

## मुख्य अवधारणाएं

### प्रोफ़ाइल

एक लॉन्च लक्ष्य जो प्रदाता मेटाडेटा + प्रमाणीकरण रणनीति (बिल्ट-इन प्रदाता, OpenRouter उपनाम, या कस्टम प्रदाता) को हल करता है।

### मोड

`~/.claudy/modes/<name>/` पर एक नामित Claude कॉन्फ़िग डायरेक्ट्री।

जब आप यह चलाते हैं:

```bash
claudy <profile> <mode> [args...]
```

Claudy सेट करता है:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

ताकि Claude मोड-विशिष्ट कॉन्फ़िग फ़ाइलें पढ़े।

मोड समर्पित Claude फ़्रेमवर्क और टूलकिट के लिए भी एक स्वाभाविक विकल्प हैं जो अपना `CLAUDE.md`, स्किल, एजेंट, या सेटिंग्स शिप करते हैं — जैसे [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code), हमारा अपना [epic-harness](https://github.com/epicsagas/epic-harness)(एक स्व-विकसित Claude Code प्लगइन), या कोई कस्टम हार्नेस। अपने डिफ़ॉल्ट कॉन्फ़िग को प्रदूषित करने के बजाय, प्रत्येक फ़्रेमवर्क को उसके अपने मोड में विलग करें:

```bash
# फ़्रेमवर्क के लिए समर्पित मोड बनाएँ
claudy mode create gstack

# फ़्रेमवर्क कॉन्फ़िग को मोड डायरेक्ट्री में कॉपी या सिमलिंक करें
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# उस फ़्रेमवर्क को सक्रिय करके Claude लॉन्च करें
claudy <profile> gstack
```

प्रत्येक मोड डायरेक्ट्री एक स्व-निहित `CLAUDE_CONFIG_DIR` है, इसलिए फ़्रेमवर्क कभी भी एक-दूसरे या आपके डिफ़ॉल्ट सेटअप के साथ संघर्ष नहीं करते।

> **[epic-harness](https://github.com/epicsagas/epic-harness) के साथ बेहतरीन।** Claudy संचालन परत संभालता है — प्रदाता स्विचिंग, कॉन्फ़िग विलगन, चैनल/एजेंट ब्रिज — जबकि epic-harness (3 कमांड, 26 ऑटो-ट्रिगर स्किल, आपके विफलता पैटर्न से स्व-विकसित) एजेंट बुद्धिमत्ता जोड़ता है। एक ही `epicsagas` परिवार; मोड में ज़िम्मेदारियों का स्पष्ट विभाजन।

<details>
<summary>कमांड संदर्भ</summary>

## कमांड संदर्भ

### मुख्य कमांड

- `claudy ls` (उपनाम: `list`): कॉन्फ़िगर/हल की गई प्रोफ़ाइल सूचीबद्ध करें।
- `claudy setup [provider]` (उपनाम: `config`): इंटरैक्टिव प्रदाता सेटअप।
- `claudy show <profile>` (उपनाम: `info`): हल की गई प्रदाता विवरण दिखाएँ।
- `claudy ping [profile]` (उपनाम: `test`): प्रदाता कनेक्टिविटी परीक्षण।
- `claudy doctor` (उपनाम: `status`): संस्करण, पथ, और प्रोफ़ाइल गणना दिखाएँ।
- `claudy sync` (उपनाम: `install`): claudy बाइनरी इंस्टॉल/सिंक्रनाइज़ करें।
- `claudy update`: claudy अपडेट करें।
- `claudy uninstall`: इंस्टॉल की गई फ़ाइलें हटाएँ।
- `claudy mode <action> [name]`: Claude कॉन्फ़िग मोड प्रबंधित करें।
- `claudy channel <subcommand>`: चैनल ब्रिज प्रबंधित करें।
- `claudy mcp`: एजेंट ब्रिज के लिए MCP सर्वर के रूप में चलाएँ।
- `claudy analytics <subcommand>`: उपयोग विश्लेषण डैशबोर्ड।
- `claudy session sanitize`: गैर-Anthropic प्रोवाइडर द्वारा लिखे गए अमान्य thinking ब्लॉक वाले सेशन को ठीक करता है।

### मोड कमांड

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

मोड नाम नियम: `[a-z0-9][a-z0-9_-]*` (`mode` आरक्षित है)।

### चैनल कमांड (वैकल्पिक ब्रिज)

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

`channel add` आपको बॉट टोकन, अनुमत उपयोगकर्ता, प्रोफ़ाइल, और मोड मैपिंग के माध्यम से मार्गदर्शन करता है।

#### समर्थित प्लेटफ़ॉर्म

| प्लेटफ़ॉर्म | इनजेशन | इंटरैक्टिव बटन | टिप्पणी |
|----------|-----------|-------------------|-------|
| Telegram | लॉन्ग-पोलिंग + वेबहुक | इनलाइन कीबोर्ड | सबसे पूर्ण |
| Slack | इवेंट सब्सक्रिप्शन वेबहुक | Block Kit एक्शन | HMAC-SHA256 सत्यापित |
| Discord | इंटरैक्शन वेबहुक | एक्शन रो कंपोनेंट | Ed25519 सत्यापित |

#### चैनल बॉट कमांड

एक बार चलने पर, बॉट चैट में इन कमांड का जवाब देता है:

- `/help` — उपलब्ध कमांड दिखाएँ
- `/cancel` — वर्तमान कार्य रद्द करें
- `/model` — Claude मॉडल बदलें (इंटरैक्टिव बटन)
- `/yolo` — ऑटो-अलाउ अनुमति टॉगल करें
- `/status` — सेशन स्थिति, प्रोफ़ाइल, मोड, git ब्रांच, और टोकन उपयोग दिखाएँ
- `/sessions` — हालिया Claude सेशन सूचीबद्ध करें (स्विच बटन सहित)
- `/projects` — प्रोजेक्ट सूचीबद्ध करें (ब्राउज़ बटन सहित)
- `/new` — नया सेशन शुरू करें
- `/history` — हालिया सेशन इतिहास दिखाएँ

Claude से सीधे बात करने के लिए कोई अन्य टेक्स्ट भेजें।

#### अनुमति प्रॉम्प्ट

जब Claude किसी टूल का उपयोग करने की स्वीकृति माँगता है (कमांड चलाना, फ़ाइल संपादित करना, आदि),
बॉट आपकी चैट में एक इंटरैक्टिव अनुमति/अस्वीकृति प्रॉम्प्ट भेजता है। बटन टैप करने पर
प्रतिक्रिया Claude को वापस भेजी जाती है और प्रोसेसिंग स्वचालित रूप से जारी रहती है।

#### सीक्रेट

चैनल क्रेडेंशियल `~/.claudy/secrets.env` में स्टोर करें (पूर्ण प्रारूप के लिए [प्रदाता क्रेडेंशियल](#परयवरण-चर) देखें):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## एजेंट MCP ब्रिज

`claudy mcp` चलाकर एक stdio-आधारित MCP सर्वर शुरू करें जो Claude Code को अन्य स्थानीय रूप से इंस्टॉल AI कोडिंग एजेंटों को कार्य सौंपने देता है।

```bash
claudy mcp run        # MCP सर्वर शुरू करें (Claude Code द्वारा कॉल किया जाता है)
claudy mcp install    # Claude Code सेटिंग्स में MCP सर्वर के रूप में पंजीकृत करें
claudy mcp uninstall  # Claude Code MCP सेटिंग्स से हटाएँ
```

`claudy mcp install` स्वचालित रूप से `~/.claude/settings.json` में पंजीकृत होता है। जब आप `claudy mode create <name>` से मोड बनाते हैं, तो यह मोड की सेटिंग्स फ़ाइल में भी पंजीकृत होता है। कोई मैनुअल कॉन्फ़िगरेशन आवश्यक नहीं।

मैन्युअली पंजीकृत करने के लिए (या प्रोजेक्ट-स्तरीय `.claude/settings.json` में):

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

Claude Code एक `ask_agent` टूल देखेगा जो सभी इंस्टॉल एजेंटों को उजागर करता है।

### उपयोग उदाहरण

एक बार पंजीकृत होने पर, Claude Code इस तरह कार्य सौंप सकता है:

```
> Ask agy to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code उपयुक्त एजेंट चुनता है, प्रॉम्प्ट पास करता है, और परिणाम लौटाता है। आप वर्किंग डायरेक्ट्री भी निर्दिष्ट कर सकते हैं:

```json
{ "agent": "agy", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### MCP पंजीकरण सत्यापित करें

```bash
# claudy पंजीकृत है या नहीं जाँचें
cat ~/.claude/settings.json | grep -A3 claudy

# MCP सर्वर मैन्युअली परीक्षण
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### समर्थित एजेंट (PATH से ऑटो-डिटेक्ट)

| Agent | Binary | Headless command |
|-------|--------|-----------------|
| Antigravity | `agy` | `agy -p "..." --output-format text` |
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

### कस्टम एजेंट

`~/.claudy/config.yaml` में `agents` कुंजी के अंतर्गत एजेंट जोड़ें (पूर्ण स्कीमा के लिए [कॉन्फ़िगरेशन](#configyaml-सकमम) देखें):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

बिल्ट-इन एजेंट के समान कुंजी उसके डिफ़ॉल्ट को ओवरराइड करती है। `args` में `{prompt}` को वास्तविक कार्य से बदल दिया जाता है।

## उपयोग विश्लेषण

> **नोट**: विश्लेषण सुविधा अभी भी काम जारी है। टोकन गणना, लागत अनुमान, और अन्य मेट्रिक्स पूरी तरह से सटीक नहीं हो सकते। आगामी रिलीज़ में सुधार की उम्मीद करें।

```bash
claudy analytics dashboard         # लोकल विश्लेषण डैशबोर्ड खोलें (Tauri 2)
claudy analytics ingest            # ~/.claude/projects/ से सेशन डेटा इनजेस्ट करें
claudy analytics ingest --full     # सभी फ़ाइलें पुनः इनजेस्ट करें (चेकपॉइंट अनदेखा)
claudy analytics ingest --project my-project  # विशिष्ट प्रोजेक्ट इनजेस्ट करें
claudy analytics recommend         # CLI में उपयोग सिफ़ारिशें दिखाएँ
claudy analytics export            # विश्लेषण डेटा निर्यात करें (JSON, डिफ़ॉल्ट 30 दिन)
claudy analytics export --format csv --days 7  # पिछले 7 दिनों का CSV निर्यात
claudy analytics sync-pricing      # models.dev और Anthropic प्राइसिंग पेज से मॉडल प्राइसिंग सिंक करें
claudy analytics recalculate       # नवीनतम प्राइसिंग डेटा से सभी लागत पुनर्गणना करें
claudy analytics insights          # संक्षिप्त JSON इंसाइट्स सारांश उत्पन्न करें (डिफ़ॉल्ट: 7 दिन)
claudy analytics insights --days 14  # पिछले 14 दिनों का विश्लेषण
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # विशिष्ट तिथि सीमा
claudy analytics insights --project my-project  # प्रोजेक्ट द्वारा फ़िल्टर
```

### Claude Code के अंदर: `/analytics-insights`

अपने उपयोग का विश्लेषण करने का सबसे तेज़ तरीका सीधे Claude Code के अंदर है। `analytics-insights` स्किल स्वचालित रूप से उपलब्ध है — बस स्वाभाविक रूप से पूछें:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> उपयोग पैटर्न का विश्लेषण करो
```

Claude `claudy analytics insights` चलाता है, JSON का विश्लेषण करता है, और एक संरचित रिपोर्ट लौटाता है:

- **लागत रुझान** — दैनिक/साप्ताहिक खर्च स्पाइक डिटेक्शन सहित
- **मॉडल वितरण** — आप कौन से मॉडल उपयोग करते हैं और प्रति सेशन उनकी लागत
- **टूल पैटर्न** — सबसे अधिक उपयोग किए जाने वाले टूल, त्रुटि दरें, दक्षता अवलोकन
- **कैश प्रदर्शन** — हिट अनुपात और अनुमानित बचत
- **कार्रवाई योग्य सिफ़ारिशें** — "सरल कार्यों को turbo पर रूट करें" जैसे विशिष्ट सुझाव अनुमानित डॉलर बचत सहित

आउटपुट उदाहरण (कच्चा डेटा देखें [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json)):

```
#### सारांश
81 सेशन, कुल $481 खर्च, औसतन $68.7/दिन। लागत तेज़ी से
बढ़ रही है — पिछले 3 कार्य दिवसों का औसत $97/दिन।

#### सिफ़ारिशें
1. सरल कार्यों को glm-5-turbo पर रूट करें — अनुमानित बचत: ~$90/माह
2. $1.91/टर्न आउटलायर सेशन की जाँच करें (औसत लागत/टर्न का 6 गुना)
3. harness ओवरहेड कम करें — TaskCreate/Update ने ~1,000 कॉल किए
```

कोई मैनुअल कमांड नहीं, कोई कॉन्टेक्स्ट स्विचिंग नहीं। Claude से अपने उपयोग के बारे में पूछें और तुरंत उत्तर पाएँ।

### विश्लेषण क्या ट्रैक करता है

- **टोकन**: पिछले 30 दिनों में इनपुट, आउटपुट, और कैश टोकन के विस्तृत रुझान, मॉडल और तिथि के अनुसार समूहबद्ध।
- **टूल**: Claude द्वारा सबसे अधिक उपयोग किए जाने वाले टूल का वितरण विश्लेषण, कॉल गणना, त्रुटि दरें, और औसत निष्पादन समय सहित।
- **लागत**: वास्तविक टोकन प्राइसिंग पर आधारित उपयोग लागत का रीयल-टाइम अनुमान, दैनिक/साप्ताहिक/मासिक पूर्वानुमान और रुझान पहचान (बढ़ता/स्थिर/घटता) सहित।
- **सुझाव (सिफ़ारिशें)**: डेटा-संचालित अनुकूलन सलाह, जैसे उच्च-लागत सेशन का पता लगाना, सरल कार्यों के लिए Haiku का सुझाव देना, और लंबी बातचीत की पहचान जो कॉन्टेक्स्ट सारांश से लाभान्वित हो सकती हैं।
- **प्रोजेक्ट**: बेहतर कॉन्टेक्स्ट के लिए क्रिप्टिक सेशन UUID को मानव-पठनीय प्रोजेक्ट फ़ोल्डर नामों में स्वचालित रूप से मैप करें।

डेटा `~/.claudy/analytics/` के अंतर्गत एक लोकल SQLite डेटाबेस में संग्रहीत होता है। डैशबोर्ड एक उच्च-प्रदर्शन लोकल Tauri 2 + Svelte ऐप के रूप में चलता है। अपने Claude CLI इतिहास से डेटा तुरंत रिफ़्रेश करने के लिए डैशबोर्ड में **[Sync]** बटन का उपयोग करें।

### विश्लेषण डैशबोर्ड
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="विश्लेषण डैशबोर्ड" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## क्रॉस-प्रोवाइडर सेशन निरंतरता

गैर-Anthropic प्रोवाइडर (जैसे Z.AI / GLM) के साथ काम करने पर, सेशन JSONL फ़ाइल में खाली signature वाले thinking ब्लॉक दर्ज होते हैं। Anthropic API से उस सेशन को फिर से शुरू करने पर यह त्रुटि आती है:

```
API Error: 400 Invalid `signature` in `thinking` block
```

Claudy इसे दो तरीकों से संभालता है:

**स्वचालित (चैनल ब्रिज):** जब चैनल सर्वर सेशन फिर से शुरू करता है, तो वह खाली signature वाले thinking ब्लॉक को चुपचाप टेक्स्ट ब्लॉक में बदल देता है। कोई कार्रवाई आवश्यक नहीं।

**मैन्युअल (CLI):** `claude --resume` से फिर से शुरू करने से पहले `claudy session sanitize` चलाएं:

```bash
# इंटरैक्टिव — समस्या वाले सेशन की सूची से चुनें
claudy session sanitize

# प्रोजेक्ट नाम से फ़िल्टर करें
claudy session sanitize --project book-forge

# सभी समस्या वाले सेशन एक साथ ठीक करें
claudy session sanitize --all --yes
```

**रूपांतरण क्या करता है:** खाली signature वाले thinking ब्लॉक को सादे टेक्स्ट ब्लॉक में फिर से लिखा जाता है। वैध Anthropic signature वाले ब्लॉक नहीं बदले जाते।

**सीमा:** सेशन निरंतरता बातचीत के इतिहास की संगतता पर निर्भर करती है। सेशन के बीच में प्रोवाइडर बदलने पर sanitization के बाद भी मामूली संदर्भ परिवर्तन हो सकते हैं।

---

## फ़ाइलें और निर्देशिका लेआउट

डिफ़ॉल्ट रूप से, Claudy डेटा यहाँ संग्रहीत करता है:

```text
~/.claudy/
```

महत्वपूर्ण फ़ाइलें/निर्देशिकाएँ:

- `config.yaml`: प्रदाता + चैनल + एजेंट कॉन्फ़िगरेशन।
- `secrets.env`: प्रदाता/बॉट क्रेडेंशियल।
- `launchers.json`: लॉन्चर/सिमलिंक मैनिफ़ेस्ट।
- `modes/`: Claude कॉन्फ़िग मोड।
- `session-patches/`: सेशन पैच स्टोरेज।
- `channel/`: चैनल रनटाइम स्थिति (`pid`, सेशन, ऑडिट लॉग)।
- `analytics/`: विश्लेषण SQLite डेटाबेस और चेकपॉइंट।
- `cache/update.json`: अपडेट मेटाडेटा कैश।

## पर्यावरण चर

- `CLAUDY_HOME`: Claudy होम डायरेक्ट्री ओवरराइड करें (डिफ़ॉल्ट: `~/.claudy`)।
- `CLAUDE_CONFIG_DIR`: मोड के साथ लॉन्च करते समय Claudy द्वारा स्वचालित रूप से सेट।

## सामान्य वर्कफ़्लो

### प्रदाता कॉन्फ़िगर और लॉन्च करें

```bash
claudy setup
claudy <profile>
```

### प्रदाता के साथ मोड उपयोग करें

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` claudy का `--dangerously-skip-permissions` का संक्षिप्त रूप है।

### समर्पित Claude फ़्रेमवर्क अपने मोड में चलाएँ

gstack, superpowers, ecc, या हमारे [epic-harness](https://github.com/epicsagas/epic-harness) जैसे फ़्रेमवर्क अपना `CLAUDE.md`, स्किल, और एजेंट शिप करते हैं। उन्हें विलग रखें:

```bash
# एक बार सेटअप: मोड बनाएँ और फ़्रेमवर्क कॉन्फ़िग से सीड करें
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# दैनिक उपयोग: फ़्रेमवर्क सक्रिय करके Claude लॉन्च करें
claudy <profile> gstack
```

अपने डिफ़ॉल्ट कॉन्फ़िग को छुए बिना फ़्रेमवर्क के बीच स्विच करें:

```bash
claudy <profile> gstack      # gstack फ़्रेमवर्क सक्रिय
claudy <profile> superpowers # superpowers फ़्रेमवर्क सक्रिय
claudy <profile>             # आपका डिफ़ॉल्ट कॉन्फ़िग, अपरिवर्तित
```

### MCP के माध्यम से अन्य एजेंटों को कार्य सौंपें

```bash
# 1) सुनिश्चित करें कि MCP पंजीकृत है (पहले `claudy mcp` पर स्वचालित)
claudy mcp

# 2) Claude Code में, किसी भी इंस्टॉल एजेंट को कार्य सौंपने को कहें:
#    "Ask agy to analyze this error"
#    "Ask aider to refactor the auth module"
```

### इंस्टॉल/कॉन्फ़िगरेशन स्थिति का निदान करें

```bash
claudy doctor
claudy ping
```

## समस्या निवारण

- **`profile not recognized`**: `claudy ls` चलाएँ और सूचीबद्ध प्रोफ़ाइल ID चुनें।
- **`not configured` प्रोफ़ाइल**: क्रेडेंशियल जोड़ने के लिए `claudy setup <provider>` चलाएँ।
- **चैनल स्थिति अस्वस्थ**: `claudy channel status` चलाएँ, फिर `claudy channel stop` और `claudy channel start` से पुनः आरंभ करें।
- **चैनल बॉट प्रतिक्रिया नहीं दे रहा**: `~/.claudy/channel/logs/server.log` में त्रुटियाँ जाँचें। `~/.claudy/secrets.env` में बॉट टोकन सत्यापित करें और जाँचें कि `allowed_users` में आपकी चैट उपयोगकर्ता ID शामिल है।
- **अनुमति प्रॉम्प्ट दिखाई नहीं दे रहा**: सुनिश्चित करें कि Claude CLI `--dangerously-skip-permissions` के साथ नहीं चल रहा है। प्रॉम्प्ट केवल तभी ट्रिगर होता है जब Claude को टूल उपयोग के लिए स्पष्ट स्वीकृति चाहिए।
- **इंस्टॉल के बाद बाइनरी नहीं मिल रही**: [सत्यापित करें](#verify) सेक्शन में PATH नोट देखें।
- **एजेंट MCP में नहीं दिख रहा**: सुनिश्चित करें कि एजेंट बाइनरी `PATH` पर है (`which agy`)। केवल इंस्टॉल एजेंट `tools/list` में दिखाई देते हैं।
- **एजेंट टाइमआउट**: `config.yaml` के agents फ़ील्ड में टाइमआउट बढ़ाएँ (डिफ़ॉल्ट: 120 सेकंड)।
- **MCP पंजीकृत नहीं**: `claudy mcp` एक बार मैन्युअली चलाएँ, या `~/.claude/settings.json` में `mcpServers.claudy` प्रविष्टि जाँचें।
- **एजेंट आउटपुट कट गया**: एजेंट stdout 10MB तक सीमित है। बड़े आउटपुट के लिए, एजेंट को फ़ाइल में लिखने के लिए रीडायरेक्ट करें।
- **विश्लेषण डेटा गायब**: `claudy analytics ingest` चलाकर `~/.claude/projects/` से डेटा भरें। सब कुछ पुनः इनजेस्ट करने के लिए `--full` उपयोग करें।
- **सेशन फिर से शुरू करने पर `400 Invalid signature in thinking block`**: यह सेशन Z.AI जैसे गैर-Anthropic प्रोवाइडर से बनाया गया था। `claudy session sanitize` चलाएं और फिर सामान्य रूप से फिर से शुरू करें।

## विकास

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# विश्लेषण बैकएंड परीक्षण (लोकल DB उपयोग)
cargo run --example test_dashboard --features analytics-ui

# विश्लेषण डैशबोर्ड लॉन्च करें (analytics-ui फ़ीचर आवश्यक)
cargo run --features analytics-ui -- analytics dashboard
```

## योगदान

योगदान का स्वागत है! आरंभ करने का तरीका:

1. रिपॉज़िटरी को फ़ोर्क करें और एक फ़ीचर ब्रांच बनाएँ।
2. उचित स्थानों पर परीक्षणों के साथ अपने परिवर्तन करें।
3. सबमिट करने से पहले `cargo test && cargo clippy -- -D warnings` चलाएँ।
4. https://github.com/epicsagas/claudy पर पुल रिक्वेस्ट खोलें।

बग रिपोर्ट और फ़ीचर अनुरोध [GitHub Issues](https://github.com/epicsagas/claudy/issues) के माध्यम से स्वागतयोग्य हैं।

## आभार

यह प्रोजेक्ट [Clother](https://github.com/jolehuit/clother) से प्रेरित है, जो Claude CLI के लिए Go-आधारित मल्टी-प्रदाता लॉन्चर है। Claudy एक स्वतंत्र Rust कार्यान्वयन है, जिसे RAII-आधारित सेशन गार्ड, सिग्नल फ़ॉरवर्डिंग, लॉन्चर सिमलिंक, और गहन इकोसिस्टम एकीकरण सहित नींव से पुनः डिज़ाइन किया गया है, जिसमें **पूर्ण-सुविधा चैनल ब्रिज** (Telegram/Slack/Discord), क्रॉस-एजेंट प्रतिनिधिमंडल के लिए **एजेंट MCP ब्रिज**, और Tauri 2 से निर्मित **उच्च-प्रदर्शन विश्लेषण डैशबोर्ड** शामिल हैं। ये additions Claudy के एक सरल लॉन्चर से Claude CLI उपयोगकर्ताओं के लिए एक व्यापक संचालन टूलकिट में परिवर्तन को दर्शाते हैं।

## लाइसेंस

[Apache-2.0](../../LICENSE)
