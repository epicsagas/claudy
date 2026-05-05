[← English](../../README.md)

<h1 align="center">claudy</h1>

<p align="center"><b>أمر واحد. أي Provider. تحكم كامل في Claude CLI.</b></p>

---

<p align="center">
توقف عن التنقل بين متغيرات البيئة وملفات الإعداد.<br/>
يتيح لك Claudy التبديل بين Anthropic وZ.AI وOpenRouter وOllama ونقاط النهاية المخصصة بأمر واحد — مع إبقاء بيانات الاعتماد وأوضاع الإعداد وأُطر عمل Claude معزولة بشكل نظيف لكل Profile.
</p>

<p align="center">
<b>متعدد الـ Providers · عزل الإعدادات · Channel bridge · جسر الوكلاء المحليين · تحليلات الاستخدام</b>
</p>

---

<p align="center"><b>مُشغِّل حديث متعدد الـ providers لـ Claude CLI.</b></p>

---

<p align="center">
يساعدك Claudy على تشغيل Claude مع providers متعددة من خلال واجهة أوامر موحدة، مع إبقاء بيانات اعتماد الـ provider وإعدادات Claude منظمة تحت دليل رئيسي واحد.
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

<div dir="rtl">

## لماذا Claudy؟

- **تشغيل متعدد الـ providers**: التبديل بين built-in و Z.AI و OpenRouter alias و Ollama ونقاط النهاية المتوافقة مع Anthropic المخصصة.
- **Config modes**: عزل إعدادات Claude (`CLAUDE.md` و `settings.json` والـ skills/plugins/agents) لكل Mode.
- **دقة Provider Profile**: توحيد built-in providers والـ providers المخصصة وأسماء OpenRouter aliases.
- **سلوك آمن للعملية**: إعادة توجيه SIGINT/SIGTERM إلى عملية Claude الفرعية.
- **تجربة مستخدم تشغيلية**: أوامر install/update/uninstall، وفحوصات الحالة، واختبارات الاتصال.
- **Channel bridge اختياري**: تشغيل جسر بوت محلي لـ Telegram و Slack و Discord مع نوافذ إذن تفاعلية.
- **Agent MCP bridge**: تفويض المهام من Claude Code إلى وكلاء AI محليين آخرين (Gemini و Codex و Aider وغيرها) عبر MCP.
- **تحليلات الاستخدام**: استيعاب بيانات الجلسة من `~/.claude/projects/`، وتتبع استخدام الرموز (tokens) والتكاليف لكل جلسة/مشروع، وعرض لوحة تحكم محلية مع توصيات.

## الـ Providers المدعومة

> استوحى Claudy الإلهام من [Clother](https://github.com/jolehuit/clother)، وهو مُشغِّل متعدد الـ providers لـ Claude CLI مبني بـ Go. Z.AI هو الـ provider الأكثر اختباراً. إذا واجهت أي مشكلة مع providers أخرى، يرجى [فتح issue](https://github.com/epicsagas/claudy/issues).

</div>

| Provider | الحالة | ملاحظات |
|---|---|---|
| Built-in (Anthropic) | ✅ مختبر | الافتراضي |
| Z.AI | ✅ مختبر | |
| OpenRouter alias | ⚠️ تجريبي | لم يُختبر بالكامل بعد — أبلغ عن المشكلات على GitHub |
| Ollama | ⚠️ تجريبي | لم يُختبر بالكامل بعد — أبلغ عن المشكلات على GitHub |
| Custom endpoint | ⚠️ تجريبي | لم يُختبر بالكامل بعد — أبلغ عن المشكلات على GitHub |

<div dir="rtl">

## المتطلبات

- macOS أو Linux
- Rust toolchain (`cargo`) للبناء/التثبيت من المصدر
- Claude CLI مثبّت ومتاح في `PATH`

## التثبيت

### التثبيت من crates.io

**ملف ثنائي جاهز (سريع، لا حاجة للتصريف)**

</div>

```
cargo install cargo-binstall
cargo binstall claudy
```

<div dir="rtl">

**أي platform — البناء من المصدر**

</div>

```
cargo install claudy
```

<div dir="rtl">

**MacOS homebrew**

</div>

```bash
brew tap epicsagas/tap
brew install claudy
```

<div dir="rtl">

### التثبيت من مصدر محلي

</div>

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

<div dir="rtl">

### التحقق

</div>

```bash
claudy --help
claudy --version
```

<div dir="rtl">

## البدء السريع

</div>

<img src="docs/assets/demo.gif" alt="Quick Start" width="100%" />

```bash
# 1) عرض قائمة الـ profiles المتاحة/المحلولة
claudy ls

# 2) إعداد بيانات الاعتماد بشكل تفاعلي
claudy setup

# 3) فحص تفاصيل Profile واحد
claudy show <profile>

# 4) تشغيل Claude مع Profile
claudy <profile> [claude-args...]
```

<div dir="rtl">

## المفاهيم الأساسية

### Profile

هدف تشغيل يحل بيانات الـ provider الوصفية + استراتيجية المصادقة (built-in provider أو OpenRouter alias أو provider مخصص).

### Mode

دليل إعداد Claude مسمى في `~/.claudy/modes/<name>/`.

عند تشغيل:

</div>

```bash
claudy <profile> <mode> [args...]
```

<div dir="rtl">

يضبط Claudy:

</div>

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

<div dir="rtl">

حتى يقرأ Claude ملفات الإعداد الخاصة بالـ Mode.

تُعدّ Modes أيضاً خياراً مثالياً لتشغيل **أُطر عمل وأدوات Claude المتخصصة** التي تأتي بـ `CLAUDE.md` وskills وagents وإعدادات خاصة بها — مثل [gstack](https://github.com/garrytan/gstack) و[superpowers](https://github.com/obra/superpowers) و[ecc](https://github.com/affaan-m/everything-claude-code) أو أي harness مخصص. بدلاً من تلويث إعداداتك الافتراضية، عزل كل إطار عمل في Mode خاص به:

</div>

```bash
# إنشاء Mode مخصص لإطار العمل
claudy mode create gstack

# نسخ أو ربط إعداد إطار العمل بدليل الـ Mode
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# تشغيل Claude مع تفعيل إطار العمل
claudy <profile> gstack
```

<div dir="rtl">

كل دليل Mode هو `CLAUDE_CONFIG_DIR` مستقل، لذا لن تتعارض أُطر العمل مع بعضها أو مع إعداداتك الافتراضية.

## مرجع الأوامر

### الأوامر الرئيسية

- `claudy ls` (alias: `list`): عرض قائمة الـ profiles المُعدَّة/المحلولة.
- `claudy setup [provider]` (alias: `config`): إعداد provider بشكل تفاعلي.
- `claudy show <profile>` (alias: `info`): عرض تفاصيل الـ provider المحلول.
- `claudy ping [profile]` (alias: `test`): اختبار اتصال الـ provider.
- `claudy doctor` (alias: `status`): عرض الإصدار والمسارات وعدد الـ profiles.
- `claudy sync` (alias: `install`): تثبيت/مزامنة الملف الثنائي claudy.
- `claudy update`: تحديث claudy.
- `claudy uninstall`: إزالة الملفات المثبتة.
- `claudy mode <action> [name]`: إدارة Claude config modes.
- `claudy channel <subcommand>`: إدارة Channel bridge.
- `claudy mcp`: التشغيل كخادم MCP لجسر الوكيل.
- `claudy analytics <subcommand>`: لوحة تحكم تحليلات الاستخدام.

### أوامر Mode

</div>

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

<div dir="rtl">

قاعدة اسم Mode: `[a-z0-9][a-z0-9_-]*` (الكلمة `mode` محجوزة).

### أوامر Channel (جسر اختياري)

</div>

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

<div dir="rtl">

`channel add` يرشدك خلال bot token والمستخدمين المسموح لهم وتعيين الـ Profile والـ Mode.

#### المنصات المدعومة

</div>

| Platform | الاستيعاب | الأزرار التفاعلية | ملاحظات |
|----------|-----------|------------------|---------|
| Telegram | Long-polling + webhook | Inline keyboard | الأكثر اكتمالاً |
| Slack | Event subscription webhook | Block Kit actions | HMAC-SHA256 موثَّق |
| Discord | Interaction webhook | Action row components | Ed25519 موثَّق |

<div dir="rtl">

#### أوامر بوت Channel

بمجرد التشغيل، يستجيب البوت لهذه الأوامر في الدردشة:

- `/help` — عرض الأوامر المتاحة
- `/cancel` — إلغاء المهمة الحالية
- `/model` — تغيير نموذج Claude (أزرار تفاعلية)
- `/yolo` — تبديل أذونات auto-allow
- `/status` — عرض حالة الجلسة والـ Profile والـ Mode وفرع git واستخدام الرموز
- `/sessions` — عرض جلسات Claude الأخيرة (مع أزرار التبديل)
- `/projects` — عرض المشاريع (مع أزرار التصفح)
- `/new` — بدء جلسة جديدة
- `/history` — عرض سجل الجلسات الأخيرة

أرسل أي نص آخر للتحدث مباشرةً مع Claude.

#### نوافذ الإذن (Permission prompts)

عندما يطلب Claude الموافقة لاستخدام أداة (تشغيل أمر، تحرير ملف، إلخ)، يرسل البوت نافذة Allow/Deny تفاعلية إلى دردشتك. يؤدي النقر على زر إلى إرسال الرد إلى Claude وتستمر المعالجة تلقائياً.

#### الـ Secrets

احفظ بيانات الاعتماد في `~/.claudy/secrets.env`:

</div>

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

<div dir="rtl">

### Agent MCP bridge

شغِّل `claudy mcp` لبدء خادم MCP قائم على stdio يتيح لـ Claude Code تفويض المهام إلى وكلاء AI برمجية محلية أخرى.

</div>

```bash
claudy mcp
```

<div dir="rtl">

عند أول تشغيل، يسجّل claudy نفسه تلقائياً في `~/.claude/settings.json`. عند إنشاء Mode بـ `claudy mode create <name>`، يسجّل أيضاً في ملف إعدادات الـ Mode. لا حاجة لأي إعداد يدوي.

للتسجيل يدوياً (أو في `.claude/settings.json` على مستوى المشروع):

</div>

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

<div dir="rtl">

سيرى Claude Code أداة `ask_agent` تعرض جميع الوكلاء المثبتين.

#### مثال على الاستخدام

بمجرد التسجيل، يمكن لـ Claude Code تفويض المهام كالتالي:

</div>

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

<div dir="rtl">

يختار Claude Code الوكيل المناسب، ويمرر الـ prompt، ويعيد النتيجة. يمكنك أيضاً تحديد دليل عمل:

</div>

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

<div dir="rtl">

#### التحقق من تسجيل MCP

</div>

```bash
# التحقق من تسجيل claudy
cat ~/.claude/settings.json | grep -A3 claudy

# اختبار خادم MCP يدوياً
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

<div dir="rtl">

#### الوكلاء المدعومون (auto-detected من PATH)

</div>

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

<div dir="rtl">

#### الوكلاء المخصصون

أضف وكلاء في `~/.claudy/config.yaml`:

</div>

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

<div dir="rtl">

نفس مفتاح الوكيل المدمج يتجاوز إعداداته الافتراضية. `{prompt}` في `args` يُستبدل بالمهمة الفعلية.

### أوامر Analytics

> **ملاحظة**: ميزة analytics لا تزال قيد التطوير. قد لا تكون أعداد الرموز وتقديرات التكلفة والمقاييس الأخرى دقيقة تماماً. توقع تحسينات في الإصدارات القادمة.

</div>

```bash
claudy analytics dashboard         # فتح لوحة تحكم analytics المحلية (Tauri 2)
claudy analytics ingest            # استيعاب بيانات الجلسة من ~/.claude/projects/
claudy analytics ingest --full     # إعادة استيعاب جميع الملفات (تجاهل checkpoints)
claudy analytics ingest --project my-project  # استيعاب مشروع محدد
claudy analytics recommend         # عرض توصيات الاستخدام في CLI
claudy analytics export            # تصدير بيانات analytics (JSON، افتراضي 30 يوماً)
claudy analytics export --format csv --days 7  # تصدير CSV لآخر 7 أيام
claudy analytics insights          # إنشاء ملخص JSON مضغوط لتحليل LLM (افتراضي: 7 أيام)
claudy analytics insights --days 14  # تحليل آخر 14 يوماً
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # نطاق تاريخ محدد
claudy analytics insights --project my-project  # تصفية حسب المشروع
```

<div dir="rtl">

تتبع Analytics:

- **Tokens**: اتجاهات تفصيلية لرموز الإدخال والإخراج والذاكرة المؤقتة خلال آخر 30 يوماً، مجمّعة حسب النموذج والتاريخ.
- **Tools**: تحليل توزيع يُظهر الأدوات التي يستخدمها Claude بأكثر تكرار، بما في ذلك أعداد الاستدعاء ومعدلات الخطأ ومتوسط وقت التنفيذ.
- **Cost**: تقدير فوري لتكاليف الاستخدام بناءً على أسعار الرموز الفعلية، بما في ذلك توقعات يومية/أسبوعية/شهرية واكتشاف الاتجاهات (increasing/stable/decreasing).
- **Tips (Recommendations)**: نصائح تحسين مدفوعة بالبيانات، كاكتشاف الجلسات عالية التكلفة، واقتراح Haiku للمهام البسيطة، وتحديد المحادثات الطويلة التي يمكن أن تستفيد من تلخيص السياق.
- **رؤى (LLM)**: ملخص استخدام مضغوط بتنسيق JSON محسّن لتحليل LLM. يجمع اتجاهات التكاليف وتوزيع النماذج وأنماط الأدوات وكفاءة التخزين المؤقت والجلسات البارزة في حمولة واحدة (~2-3K رمز). قابل للاستخدام عبر مهارة `analytics-insights` في Claude Code بلغة طبيعية — يُنشئ Claude توصيات مخصصة.
- **Projects**: يُعيّن تلقائياً معرفات UUID الغامضة للجلسات إلى أسماء مجلدات المشاريع القابلة للقراءة لتوفير سياق أفضل.

تُخزَّن البيانات في قاعدة بيانات SQLite محلية تحت `~/.claudy/analytics/`. تعمل لوحة التحكم كتطبيق Tauri 2 + Svelte محلي عالي الأداء. استخدم زر **[Sync]** في لوحة التحكم لتحديث البيانات فوراً من سجل Claude CLI.

</div>

<img src="../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

<div dir="rtl">

## الملفات وتخطيط الدليل

بشكل افتراضي، يخزن Claudy البيانات تحت:

</div>

```text
~/.claudy/
```

<div dir="rtl">

الملفات/الأدلة المهمة:

- `config.yaml`: إعداد provider + channel + agent.
- `secrets.env`: بيانات اعتماد provider/bot.
- `launchers.json`: بيان launcher/symlink.
- `modes/`: Claude config modes.
- `session-patches/`: تخزين رقع الجلسة.
- `channel/`: حالة تشغيل channel (`pid` والجلسات وسجل التدقيق).
- `analytics/`: قاعدة بيانات SQLite للتحليلات والـ checkpoints.
- `cache/update.json`: ذاكرة تخزين مؤقت لبيانات التحديث.

## متغيرات البيئة

- `CLAUDY_HOME`: تجاوز دليل Claudy الرئيسي (الافتراضي: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: يُضبط تلقائياً بواسطة Claudy عند التشغيل مع Mode.

## سير العمل الشائعة

### إعداد provider وتشغيله

</div>

```bash
claudy setup
claudy <profile>
```

<div dir="rtl">

### استخدام Mode مع provider

</div>

```bash
claudy mode create work
claudy <profile> work --yolo
```

<div dir="rtl">

> `--yolo` هو اختصار claudy لـ `--dangerously-skip-permissions`.

### تشغيل إطار عمل Claude مخصص في Mode خاص به

أُطر العمل مثل gstack وsuperpowers وecc تأتي بـ `CLAUDE.md` وskills وagents خاصة بها. شغّلها بشكل معزول:

</div>

```bash
# إعداد لمرة واحدة: إنشاء Mode وتحميل إعدادات إطار العمل
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# الاستخدام اليومي: تشغيل Claude مع تفعيل إطار العمل
claudy <profile> gstack
```

<div dir="rtl">

التبديل بين أُطر العمل دون تعديل الإعدادات الافتراضية:

</div>

```bash
claudy <profile> gstack      # إطار عمل gstack مفعّل
claudy <profile> superpowers # إطار عمل superpowers مفعّل
claudy <profile>             # الإعدادات الافتراضية، دون تغيير
```

<div dir="rtl">

### تفويض المهام إلى وكلاء آخرين عبر MCP

</div>

```bash
# 1) تأكد من تسجيل MCP (يحدث تلقائياً عند أول `claudy mcp`)
claudy mcp

# 2) في Claude Code، اطلب منه التفويض لأي وكيل مثبت:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

<div dir="rtl">

### تشخيص حالة التثبيت/الإعداد

</div>

```bash
claudy doctor
claudy ping
```

<div dir="rtl">

## استكشاف الأخطاء وإصلاحها

- **`profile not recognized`**: شغِّل `claudy ls` واختر معرف Profile المُدرج.
- **Profile بحالة `not configured`**: شغِّل `claudy setup <provider>` لإضافة بيانات الاعتماد.
- **Channel status غير صحي**: شغِّل `claudy channel status`، ثم أعد التشغيل بـ `claudy channel stop` و `claudy channel start`.
- **بوت Channel لا يستجيب**: تحقق من `~/.claudy/channel/logs/server.log` للأخطاء. تحقق من bot token في `~/.claudy/secrets.env` وأن `allowed_users` يتضمن معرف مستخدم الدردشة الخاص بك.
- **Permission prompt لا يظهر**: تأكد من أن Claude CLI لا يعمل بـ `--dangerously-skip-permissions`. تُشغَّل النافذة فقط عندما يحتاج Claude إلى موافقة صريحة لاستخدام الأداة.
- **الملف الثنائي غير موجود بعد التثبيت**: تأكد من أن دليل bin الخاص بـ Claudy موجود في `PATH`، ثم أعد تشغيل shell.
- **الوكيل لا يظهر في MCP**: تأكد من أن الملف الثنائي للوكيل موجود في `PATH` (`which gemini`). تظهر فقط الوكلاء المثبتة في `tools/list`.
- **Agent timeout**: زِد مهلة الانتظار في حقل agents في `config.yaml` (الافتراضي: 120s).
- **MCP غير مسجل**: شغِّل `claudy mcp` مرة واحدة يدوياً، أو تحقق من إدخال `mcpServers.claudy` في `~/.claude/settings.json`.
- **إخراج الوكيل مقطوع**: إخراج stdout للوكيل محدود بـ 10MB. للمخرجات الكبيرة، وجِّه الوكيل للكتابة إلى ملف بدلاً من ذلك.
- **بيانات Analytics مفقودة**: شغِّل `claudy analytics ingest` للتعبئة من `~/.claude/projects/`. استخدم `--full` لإعادة استيعاب كل شيء.

## التطوير

</div>

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# اختبار analytics backend (يستخدم قاعدة بيانات محلية)
cargo run --example test_dashboard --features analytics-ui

# تشغيل analytics dashboard (يتطلب ميزة analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

<div dir="rtl">

## المساهمة

نرحب بالمساهمات! إليك كيفية البدء:

1. Fork المستودع وأنشئ فرع ميزة.
2. أجرِ تغييراتك مع اختبارات حيثما يكون ذلك مناسباً.
3. شغِّل `cargo test && cargo clippy -- -D warnings` قبل التقديم.
4. افتح Pull Request على https://github.com/epicsagas/claudy.

تقارير الأخطاء وطلبات الميزات مرحب بها عبر [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## شكر وتقدير

استوحى هذا المشروع الإلهام من [Clother](https://github.com/jolehuit/clother)، وهو مُشغِّل متعدد الـ providers لـ Claude CLI مبني بـ Go. Claudy هو تطبيق Rust مستقل، أُعيد تصميمه من الصفر مع RAII-based session guards وإعادة توجيه الإشارات وروابط المُشغِّل وتكاملات النظام البيئي العميقة، بما في ذلك **Channel Bridge كامل المزايا** (Telegram/Slack/Discord)، و**Agent MCP Bridge** للتفويض بين الوكلاء، و**لوحة تحكم Analytics عالية الأداء** مبنية بـ Tauri 2. تعكس هذه الإضافات انتقال Claudy من مُشغِّل بسيط إلى مجموعة أدوات تشغيلية شاملة لمستخدمي Claude CLI.

## الرخصة

[Apache-2.0](../../LICENSE)

</div>
