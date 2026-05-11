<h1 align="center">claudy</h1>

<p align="center"><b>أمر واحد. أي مزوّد. تحكّم كامل في Claude CLI.</b></p>

<p align="center">
توقف عن التعامل مع متغيرات البيئة وملفات الإعداد.<br/>
يتيح لك Claudy التبديل بين Anthropic وZ.AI وOpenRouter وOllama والنقاط النهائية المخصصة بأمر واحد — مع الحفاظ على بيانات الاعتماد وأوضاع الإعداد وأُطر Claude معزولة نظيفًا لكل ملف شخصي.
</p>

<p align="center">
<b>مزوّدون متعددون · عزل الإعداد · جسر القنوات · جسر الوكلاء المحلي · تحليلات الاستخدام</b>
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
  <img alt="لماذا Claudy" src="../assets/features-2048.png" width="100%">
</picture>

## لماذا Claudy

| | الميزة | لماذا هي مهمة |
|--|---------|---------------|
| 🔄 | تشغيل متعدّد المزوّدين | التبديل بين Anthropic وZ.AI وOpenRouter وOllama والنقاط النهائية المخصصة بأمر واحد |
| 📦 | أوضاع الإعداد | عزل `CLAUDE.md` والإعدادات والمهارات والوكلاء لكل وضع — بدون تلوث متبادل |
| 🔗 | جسر الوكلاء MCP | تفويض المهام من Claude Code إلى Gemini وCodex وAider وأكثر من 20 وكيلاً آخر |
| 💬 | جسر القنوات | تشغيل بوتات Telegram وSlack وDiscord مع مطالبات أذونات تفاعلية |
| 📊 | تحليلات الاستخدام | تتبّع استخدام التوكنات والتكاليف وأنماط الأدوات عبر لوحة تحكم Tauri محلية |
| 🔐 | تحكّم آمن بالعمليات | تمرير SIGINT/SIGTERM، كتابة إعدادات ذرية، تخزين بيانات اعتماد بصلاحية 0600 |
| 🛠️ | تجربة تشغيلية | تثبيت، تحديث، إزالة، تشخيص، اختبار اتصال — كل شيء من ملف ثنائي واحد |

## مزوّدو الخدمة المدعومون

> استوحِي Claudy من [Clother](https://github.com/jolehuit/clother)، وهو مشغّل متعدّد المزوّدين مكتوب بلغة Go لـ Claude CLI. يعدّ Z.AI المزوّد الأكثر اختبارًا بشكل شامل. إذا واجهت أي مشاكل مع المزوّدين الآخرين، يُرجى [فتح مشكلة](https://github.com/epicsagas/claudy/issues).

| المزوّد | الحالة | ملاحظات |
|---|---|---|
| مدمج (Anthropic) | ✅ مُختبر | الافتراضي |
| Z.AI | ✅ مُختبر | |
| اسم مستعار OpenRouter | ⚠️ تجريبي | غير مُختبر بالكامل — أبلغ عن المشاكل على GitHub |
| Ollama | ⚠️ تجريبي | غير مُختبر بالكامل — أبلغ عن المشاكل على GitHub |
| نقطة نهائية مخصصة | ⚠️ تجريبي | غير مُختبر بالكامل — أبلغ عن المشاكل على GitHub |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="عرض توضيحي" src="../assets/demo.gif" width="100%">
</picture>

## البدء السريع

**1. التثبيت**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

لا تملك Homebrew؟ استخدم نص التثبيت:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

عبر سلسلة أدوات Rust:

```bash
cargo binstall claudy   # ملف ثنائي مُجمّع مسبقًا (سريع)
cargo install claudy    # بناء من المصدر
```

**2. الإعداد**

```bash
claudy install                        # تهيئة الأدلة والإعداد وبيانات الاعتماد
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. التشغيل**

```bash
claudy                                # المزوّد الافتراضي
claudy zai                            # مزوّد Z.AI
claudy openrouter sonnet              # اسم مستعار OpenRouter
```

**4. التحديث**

```bash
brew upgrade claudy          # Homebrew
claudy update                # المحدّث المدمج
# أو أعد تشغيل نص التثبيت / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>بيانات اعتماد المزوّدين</summary>

| المتغيّر | المزوّد |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (أصلي) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI الصين |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax الصين |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `ALIBABA_API_KEY` | Alibaba Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (جميع الأسماء المستعارة) |

تستخدم المزوّدات المخصصة متغيّر `api_key_env` المعرّف في إدخال `custom_providers` الخاص بها.

</details>

<details>
<summary>مخطط config.yaml</summary>

جميع الإعدادات موجودة في `~/.claudy/config.yaml`. أضف فقط الأقسام التي تحتاجها — يتم استخدام القيم الافتراضية لأي شيء محذوف.

```yaml
# تجاوزات المزوّد — تجاوز النموذج الافتراضي ومستويات النماذج لكل مزوّد
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"                # → ANTHROPIC_DEFAULT_HAIKU_MODEL
      sonnet: "glm-5.1"               # → ANTHROPIC_DEFAULT_SONNET_MODEL
      opus: "glm-5"                   # → ANTHROPIC_DEFAULT_OPUS_MODEL

# أسماء OpenRouter المستعارة — الاستخدام: claudy or <alias>
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

# مزوّدون مخصصون متوافقون مع Anthropic — الاستخدام: claudy <slug>
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

# سياسة الضغط
compaction:
  auto_compact: true                   # الافتراضي: true
  threshold: 0.8                       # 0.0–1.0، الافتراضي: 0.8

# تجاوزات نافذة السياق لكل نموذج
model_settings:
  deepseek-chat:
    max_context_tokens: 64000

# جسر القنوات — بديل غير تفاعلي لأمر `claudy channel add`
channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0           # 0 = بلا حد
  stream_timeout_secs: 1800

# تجاوزات الوكلاء
agents:
  aider:
    binary: "aider"
    args: ["--message", "{prompt}"]
    timeout: 300
```

</details>

---

## المفاهيم الأساسية

### الملف الشخصي

هدف تشغيل يقوم بحل البيانات الوصفية للمزوّد + استراتيجية المصادقة (مزوّد مدمج، أو اسم مستعار OpenRouter، أو مزوّد مخصص).

### الوضع

دليل إعداد Claude مسمّى في `~/.claudy/modes/<name>/`.

عند تشغيل:

```bash
claudy <profile> <mode> [args...]
```

يُعيّن Claudy:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

بحيث يقرأ Claude ملفات الإعداد الخاصة بالوضع.

كما أن الأوضاع مناسبة بشكل طبيعي لـ **أُطر Claude وأدواته المخصصة** التي تأتي مع `CLAUDE.md` ومهارات ووكلاء وإعدادات خاصة بها — مثل [gstack](https://github.com/garrytan/gstack) و[superpowers](https://github.com/obra/superpowers) و[ecc](https://github.com/affaan-m/everything-claude-code) أو أي نظام مخصص. بدلاً من تلويث إعداداتك الافتراضية، اعزل كل إطار في وضعه الخاص:

```bash
# إنشاء وضع مخصص للإطار
claudy mode create gstack

# نسخ أو ربط إعداد الإطار بدليل الوضع
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# تشغيل Claude مع تفعيل ذلك الإطار
claudy <profile> gstack
```

كل دليل وضع هو `CLAUDE_CONFIG_DIR` مستقل بذاته، لذلك لن تتعارض الأُطر مع بعضها أو مع إعداداتك الافتراضية أبدًا.

<details>
<summary>مرجع الأوامر</summary>

## مرجع الأوامر

### الأوامر الرئيسية

- `claudy ls` (الاسم المستعار: `list`): عرض الملفات الشخصية المُعدّة/المحلولة.
- `claudy setup [provider]` (الاسم المستعار: `config`): إعداد المزوّد بشكل تفاعلي.
- `claudy show <profile>` (الاسم المستعار: `info`): عرض تفاصيل المزوّد المحلولة.
- `claudy ping [profile]` (الاسم المستعار: `test`): اختبار اتصال المزوّد.
- `claudy doctor` (الاسم المستعار: `status`): عرض الإصدار والمسارات وعدد الملفات الشخصية.
- `claudy sync` (الاسم المستعار: `install`): تثبيت/مزامنة الملف الثنائي claudy.
- `claudy update`: تحديث claudy.
- `claudy uninstall`: إزالة الملفات المثبتة.
- `claudy mode <action> [name]`: إدارة أوضاع إعداد Claude.
- `claudy channel <subcommand>`: إدارة جسر القنوات.
- `claudy mcp`: التشغيل كخادم MCP لجسر الوكلاء.
- `claudy analytics <subcommand>`: لوحة تحليلات الاستخدام.

### أوامر الوضع

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

قاعدة تسمية الوضع: `[a-z0-9][a-z0-9_-]*` (`mode` محجوز).

### أوامر القنوات (جسر اختياري)

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

يرشدك `channel add` خلال إعداد رمز البوت والمستخدمين المسموح لهم والملف الشخصي وتعيين الوضع.

#### المنصات المدعومة

| المنصة | الاستيعاب | الأزرار التفاعلية | ملاحظات |
|----------|-----------|-------------------|-------|
| Telegram | استطلاع طويل + webhook | لوحة مفاتيح مضمّنة | الأكثر اكتمالاً |
| Slack | اشتراك أحداث webhook | إجراءات Block Kit | مُتحقق منها عبر HMAC-SHA256 |
| Discord | webhook التفاعلات | مكونات صف الإجراءات | مُتحقق منها عبر Ed25519 |

#### أوامر بوت القناة

عند التشغيل، يستجيب البوت لهذه الأوامر في الدردشة:

- `/help` — عرض الأوامر المتاحة
- `/cancel` — إلغاء المهمة الحالية
- `/model` — تغيير نموذج Claude (أزرار تفاعلية)
- `/yolo` — تبديل الموافقة التلقائية على الأذونات
- `/status` — عرض حالة الجلسة والملف الشخصي والوضع وفرع git واستخدام التوكنات
- `/sessions` — عرض الجلسات الأخيرة (مع أزرار التبديل)
- `/projects` — عرض المشاريع (مع أزرار التصفح)
- `/new` — بدء جلسة جديدة
- `/history` — عرض سجل الجلسات الأخيرة

أرسل أي نص آخر للتحدث مباشرة مع Claude.

#### مطالبات الأذونات

عندما يطلب Claude الموافقة على استخدام أداة (تشغيل أمر، تعديل ملف، إلخ)،
يرسل البوت مطالبة تفاعلية بالسماح/الرفض إلى دردشتك. النقر على زر
يرسل الرد إلى Claude وتستمر المعالجة تلقائيًا.

#### الأسرار

خزّن بيانات اعتماد القناة في `~/.claudy/secrets.env` (راجع [بيانات اعتماد المزوّدين](#provider-credentials-secretsenv) للتنسيق الكامل):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## جسر MCP للوكلاء

شغّل `claudy mcp` لبدء خادم MCP قائم على stdio يتيح لـ Claude Code تفويض المهام إلى وكلاء برمجة AI محليين مثبتين آخرين.

```bash
claudy mcp run        # بدء خادم MCP (يُستدعى بواسطة Claude Code)
claudy mcp install    # تسجيل claudy كخادم MCP في إعدادات Claude Code
claudy mcp uninstall  # إزالة claudy من إعدادات MCP لـ Claude Code
```

يُسجّل `claudy mcp install` نفسه تلقائيًا في `~/.claude/settings.json`. عند إنشاء وضع باستخدام `claudy mode create <name>`، يُسجّل أيضًا في ملف إعدادات الوضع. لا حاجة إلى إعداد يدوي.

للتسجيل يدويًا (أو في ملف `.claude/settings.json` على مستوى المشروع):

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

سيرى Claude Code أداة `ask_agent` التي تعرض جميع الوكلاء المثبتين.

### مثال على الاستخدام

بعد التسجيل، يمكن لـ Claude Code تفويض المهام مثل هذا:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

يختار Claude Code الوكيل المناسب، ويمرّر الموجه، ويُعيد النتيجة. يمكنك أيضًا تحديد دليل عمل:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### التحقق من تسجيل MCP

```bash
# التحقق من تسجيل claudy
cat ~/.claude/settings.json | grep -A3 claudy

# اختبار خادم MCP يدويًا
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### الوكلاء المدعومون (يُكتشفون تلقائيًا من PATH)

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

### وكلاء مخصصون

أضف وكلاء في `~/.claudy/config.yaml` تحت مفتاح `agents` (راجع [الإعداد](#configyaml-schema) للمخطط الكامل):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

نفس المفتاح كوكيل مدمج يتجاوز إعداداته الافتراضية. يُستبدل `{prompt}` في `args` بالمهمة الفعلية.

## تحليلات الاستخدام

> **ملاحظة**: ميزة التحليلات لا تزال قيد التطوير. قد لا تكون أعداد التوكنات وتقديرات التكاليف والمقاييس الأخرى دقيقة بالكامل. يُرجى توقع تحسينات في الإصدارات القادمة.

```bash
claudy analytics dashboard         # فتح لوحة التحليلات المحلية (Tauri 2)
claudy analytics ingest            # استيعاب بيانات الجلسة من ~/.claude/projects/
claudy analytics ingest --full     # إعادة استيعاب جميع الملفات (تجاهل نقاط الفحص)
claudy analytics ingest --project my-project  # استيعاب مشروع محدد
claudy analytics recommend         # عرض توصيات الاستخدام في سطر الأوامر
claudy analytics export            # تصدير بيانات التحليلات (JSON، الافتراضي 30 يومًا)
claudy analytics export --format csv --days 7  # تصدير كـ CSV لآخر 7 أيام
claudy analytics sync-pricing      # مزامنة تسعير النماذج من models.dev وصفحة تسعير Anthropic
claudy analytics recalculate       # إعادة حساب جميع التكاليف باستخدام أحدث بيانات التسعير
claudy analytics insights          # إنشاء ملخص رؤى JSON مختصر (الافتراضي: 7 أيام)
claudy analytics insights --days 14  # تحليل آخر 14 يومًا
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # نطاق تاريخ محدد
claudy analytics insights --project my-project  # تصفية حسب المشروع
```

### داخل Claude Code: `/analytics-insights`

أسرع طريقة لتحليل استخدامك هي مباشرة داخل Claude Code. مهارة `analytics-insights` متاحة تلقائيًا — فقط اسأل بشكل طبيعي:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> استخدام 패턴 분석해줘
```

يشغّل Claude أمر `claudy analytics insights`، ويحلل JSON، ويُعيد تقريرًا مُنظّمًا يتضمن:

- **اتجاهات التكلفة** — الإنفاق اليومي/الأسبوعي مع كشف الارتفاعات المفاجئة
- **توزيع النماذج** — النماذج التي تستخدمها وتكلفتها لكل جلسة
- **أنماط الأدوات** — الأدوات الأكثر استخدامًا، ومعدلات الخطأ، وملاحظات الكفاءة
- **أداء التخزين المؤقت** — نسبة الإصابة والتوفير المُقدّر
- **توصيات قابلة للتنفيذ** — اقتراحات محددة مثل "وجّه المهام البسيطة إلى turbo" مع التوفير المُقدّر بالدولار

مثال على المخرجات (راجع [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json) للبيانات الخام):

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

بدون أوامر يدوية، بدون تبديل السياق. اسأل Claude عن استخدامك واحصل على إجابات فورًا.

### ما تتبّعه التحليلات

- **التوكنات**: اتجاهات تفصيلية لتوكنات الإدخال والإخراج والتخزين المؤقت خلال آخر 30 يومًا، مُجمّعة حسب النموذج والتاريخ.
- **الأدوات**: تحليل التوزيع يُظهر الأدوات التي يستخدمها Claude بشكل متكرر، بما في ذلك أعداد الاستدعاءات ومعدلات الخطأ ومتوسط وقت التنفيذ.
- **التكلفة**: تقدير فوري لتكاليف الاستخدام بناءً على تسعير التوكنات الفعلي، بما في ذلك التوقعات اليومية/الأسبوعية/الشهرية وكشف الاتجاهات (متزايد/مستقر/متناقص).
- **نصائح (توصيات)**: نصائح تحسين مبنية على البيانات، مثل كشف الجلسات عالية التكلفة، واقتراح Haiku للمهام البسيطة، وتحديد المحادثات الطويلة التي قد تستفيد من تلخيص السياق.
- **المشاريع**: تعيين معرّفات UUID الغامضة للجلسات تلقائيًا إلى أسماء مجلدات مشاريع مقروءة بشريًا لسياق أفضل.

تُخزّن البيانات في قاعدة بيانات SQLite محلية تحت `~/.claudy/analytics/`. تعمل لوحة التحكم كتطبيق محلي عالي الأداء مبني بـ Tauri 2 + Svelte. استخدم زر **[Sync]** في لوحة التحكم لتحديث البيانات فورًا من سجل Claude CLI الخاص بك.

### لوحة تحكم التحليلات
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="لوحة تحكم التحليلات" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## الملفات وتخطيط الدلائل

بشكل افتراضي، يُخزّن Claudy البيانات تحت:

```text
~/.claudy/
```

الملفات/الدلائل المهمة:

- `config.yaml`: إعداد المزوّدين + القنوات + الوكلاء.
- `secrets.env`: بيانات اعتماد المزوّدين/البوتات.
- `launchers.json`: بيان المشغّلات/الروابط الرمزية.
- `modes/`: أوضاع إعداد Claude.
- `session-patches/`: تخزين تصحيحات الجلسات.
- `channel/`: حالة تشغيل القنوات (`pid`، جلسات، سجل التدقيق).
- `analytics/`: قاعدة بيانات SQLite للتحليلات ونقاط الفحص.
- `cache/update.json`: ذاكرة التخزين المؤقت لبيانات التحديث.

## متغيرات البيئة

- `CLAUDY_HOME`: تجاوز الدليل الرئيسي لـ Claudy (الافتراضي: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: يُعيّن تلقائيًا بواسطة Claudy عند التشغيل مع وضع.

## سير العمل الشائع

### إعداد وتشغيل مزوّد

```bash
claudy setup
claudy <profile>
```

### استخدام وضع مع مزوّد

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` هو اختصار claudy لـ `--dangerously-skip-permissions`.

### تشغيل إطار Claude مخصص في وضعه الخاص

تأتي أُطر مثل gstack وsuperpowers وecc مع `CLAUDE.md` ومهارات ووكلاء خاصين بها. أبقِها معزولة:

```bash
# إعداد لمرة واحدة: إنشاء الوضع وبذره بإعداد الإطار
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# الاستخدام اليومي: تشغيل Claude مع تفعيل الإطار
claudy <profile> gstack
```

التبديل بين الأُطر دون المساس بإعداداتك الافتراضية:

```bash
claudy <profile> gstack      # إطار gstack مفعّل
claudy <profile> superpowers # إطار superpowers مفعّل
claudy <profile>             # إعداداتك الافتراضية، دون تغيير
```

### تفويض المهام إلى وكلاء آخرين عبر MCP

```bash
# 1) التأكد من تسجيل MCP (يحدث تلقائيًا عند أول `claudy mcp`)
claudy mcp

# 2) في Claude Code، اطلب منه التفويض إلى أي وكيل مثبت:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### تشخيص حالة التثبيت/الإعداد

```bash
claudy doctor
claudy ping
```

## استكشاف الأخطاء وإصلاحها

- **`profile not recognized`**: شغّل `claudy ls` واختر معرّف ملف شخصي من القائمة.
- **ملف شخصي `not configured`**: شغّل `claudy setup <provider>` لإضافة بيانات الاعتماد.
- **حالة القناة غير سليمة**: شغّل `claudy channel status`، ثم أعد التشغيل باستخدام `claudy channel stop` و`claudy channel start`.
- **بوت القناة لا يستجيب**: تحقق من `~/.claudy/channel/logs/server.log` بحثًا عن أخطاء. تحقق من رمز البوت في `~/.claudy/secrets.env` وأن `allowed_users` يتضمن معرّف مستخدم الدردشة الخاص بك.
- **مطالبة الأذونات لا تظهر**: تأكد من أن Claude CLI لا يعمل مع `--dangerously-skip-permissions`. لا تظهر المطالبة إلا عندما يحتاج Claude إلى موافقة صريحة على استخدام الأدوات.
- **الملف الثنائي غير موجود بعد التثبيت**: راجع ملاحظة PATH في قسم [التحقق](#verify).
- **الوكيل لا يظهر في MCP**: تأكد من أن الملف الثنائي للوكيل على `PATH` (`which gemini`). فقط الوكلاء المثبتون يظهرون في `tools/list`.
- **انتهاء مهلة الوكيل**: زيّد المهلة في حقل agents في `config.yaml` (الافتراضي: 120 ثانية).
- **MCP غير مسجّل**: شغّل `claudy mcp` يدويًا مرة واحدة، أو تحقق من `~/.claude/settings.json` بحثًا عن إدخال `mcpServers.claudy`.
- **مخرجات الوكيل مقطوعة**: الإخراج القياسي للوكيل محدود بـ 10 ميجابايت. للمخرجات الكبيرة، وجّه الوكيل للكتابة في ملف بدلاً من ذلك.
- **بيانات التحليلات مفقودة**: شغّل `claudy analytics ingest` لملئها من `~/.claudy/projects/`. استخدم `--full` لإعادة استيعاب كل شيء.

## التطوير

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# اختبار واجهة التحليلات (تستخدم قاعدة بيانات محلية)
cargo run --example test_dashboard --features analytics-ui

# تشغيل لوحة تحكم التحليلات (تتطلب ميزة analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## المساهمة

المساهمات مرحب بها! إليك كيف تبدأ:

1. قم بعمل Fork للمستودع وأنشئ فرع ميزة.
2. أجرِ تغييراتك مع اختبارات حيثما يناسب.
3. شغّل `cargo test && cargo clippy -- -D warnings` قبل الإرسال.
4. افتح طلب سحب على https://github.com/epicsagas/claudy.

نرحب بتقارير الأخطاء وطلبات الميزات عبر [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## شكر وتقدير

استُوحي هذا المشروع من [Clother](https://github.com/jolehuit/clother)، وهو مشغّل متعدّد المزوّدين مكتوب بلغة Go لـ Claude CLI. Claudy هو تطبيق مستقل بلغة Rust، أُعيد تصميمه من الصفر مع حراس جلسات قائمين على RAII، وتمرير الإشارات، وروابط مشغّلات رمزية، وتكاملات عميقة مع النظام البيئي تشمل **جسر قنوات كامل الميزات** (Telegram/Slack/Discord)، و**جسر MCP للوكلاء** للتفويض بين الوكلاء، و**لوحة تحليلات عالية الأداء** مبنية بـ Tauri 2. تعكس هذه الإضافات تحوّل Claudy من مشغّل بسيط إلى مجموعة أدوات تشغيلية شاملة لمستخدمي Claude CLI.

## الترخيص

[Apache-2.0](../../LICENSE)
