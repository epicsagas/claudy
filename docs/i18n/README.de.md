<h1 align="center">claudy</h1>

<p align="center"><b>Ein Befehl. Jeder Anbieter. Volle Kontrolle über die Claude CLI.</b></p>

<p align="center">
Schluss mit dem Jonglieren von Umgebungsvariablen und Konfigurationsdateien.<br/>
Mit Claudy wechseln Sie zwischen Anthropic, Z.AI, OpenRouter, Ollama und benutzerdefinierten Endpunkten mit einem einzigen Befehl — Zugangsdaten, Konfigurationsmodi und Claude-Frameworks bleiben sauber pro Profil isoliert.
</p>

<p align="center">
<b>Mehrere Anbieter · Konfigurationsisolation · Channel-Bridge · Lokale Agent-Bridge · Nutzungsanalyse</b>
</p>

---

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> •
  <a href="README.ko.md">🇰🇷 한국어</a> •
  <a href="README.zh-Hans.md">🇨🇳 中文</a> •
  <a href="README.ja.md">🇯🇵 日本語</a> •
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
  <img alt="Warum Claudy" src="../assets/features-2048.png" width="100%">
</picture>

## Warum Claudy

| | Funktion | Warum es wichtig ist |
|--|---------|----------------|
| 🔄 | Multi-Anbieter-Start | Wechseln Sie zwischen Anthropic, Z.AI, OpenRouter, Ollama und benutzerdefinierten Endpunkten mit einem Befehl |
| 📦 | Konfigurationsmodi | `CLAUDE.md`, Einstellungen, Skills und Agents pro Modus isolieren — keine Kreuzkontamination |
| 🔗 | Agent-MCP-Bridge | Aufgaben von Claude Code an Gemini, Codex, Aider und 20+ weitere Agents delegieren |
| 💬 | Channel-Bridge | Telegram-, Slack- und Discord-Bots mit interaktiven Berechtigungsabfragen betreiben |
| 📊 | Nutzungsanalyse | Token-Verbrauch, Kosten und Tool-Muster mit einem lokalen Tauri-Dashboard verfolgen |
| 🔐 | Sicherer Prozesskontrolle | SIGINT/SIGTERM-Weiterleitung, atomare Konfigurationsschreibvorgänge, 0600-Zugangsdatenspeicherung |
| 🛠️ | Operationale UX | Installieren, Aktualisieren, Deinstallieren, Doctor, Ping — alles aus einer Binärdatei |

## Unterstützte Anbieter

> Claudy wurde inspiriert von [Clother](https://github.com/jolehuit/clother), einem Go-basierten Multi-Anbieter-Launcher für die Claude CLI. Z.AI ist der am gründlichsten getestete Anbieter. Wenn Sie Probleme mit anderen Anbietern haben, bitte [ein Issue öffnen](https://github.com/epicsagas/claudy/issues).

| Anbieter | Status | Hinweise |
|---|---|---|
| Eingebaut (Anthropic) | ✅ Getestet | Standard |
| Z.AI | ✅ Getestet | |
| OpenRouter-Alias | ⚠️ Experimentell | Nicht vollständig getestet — Probleme auf GitHub melden |
| Ollama | ⚠️ Experimentell | Nicht vollständig getestet — Probleme auf GitHub melden |
| Benutzerdefinierter Endpunkt | ⚠️ Experimentell | Nicht vollständig getestet — Probleme auf GitHub melden |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="Demo" src="../assets/demo.gif" width="100%">
</picture>

## Schnellstart

**1. Installieren**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

Kein Homebrew? Das Installationsprogramm verwenden:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Über Rust-Toolchain:

```bash
cargo binstall claudy   # vorgefertigte Binärdatei (schnell)
cargo install claudy    # aus Quellcode erstellen
```

**2. Konfigurieren**

```bash
claudy install                        # Verzeichnisse, Konfiguration, Zugangsdaten initialisieren
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. Starten**

```bash
claudy                                # Standardanbieter
claudy zai                            # Z.AI-Anbieter
claudy openrouter sonnet              # OpenRouter-Alias
```

**4. Aktualisieren**

```bash
brew upgrade claudy          # Homebrew
claudy update                # integrierter Updater
# oder das Installationsskript erneut ausführen / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>Anbieter-Anmeldeinformationen</summary>

| Variable | Anbieter |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (nativ) |
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
| `OPENROUTER_API_KEY` | OpenRouter (alle Aliase) |

Benutzerdefinierte Anbieter verwenden die `api_key_env`-Variable, die in ihrem `custom_providers`-Eintrag definiert ist.

</details>

<details>
<summary>config.yaml-Schema</summary>

Die gesamte Konfiguration befindet sich in `~/.claudy/config.yaml`. Nur die benötigten Abschnitte hinzufügen — für alles Ausgelassene werden Standardwerte verwendet.

```yaml
# Anbieter-Overrides — Standardmodell und Modell-Tiers pro Anbieter überschreiben
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"                # → ANTHROPIC_DEFAULT_HAIKU_MODEL
      sonnet: "glm-5.1"               # → ANTHROPIC_DEFAULT_SONNET_MODEL
      opus: "glm-5"                   # → ANTHROPIC_DEFAULT_OPUS_MODEL

# OpenRouter-Aliase — aufrufen als: claudy or <alias>
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

# Benutzerdefinierte Anthropic-kompatible Anbieter — aufrufen als: claudy <slug>
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

# Kompaktierungsrichtlinie
compaction:
  auto_compact: true                   # Standard: true
  threshold: 0.8                       # 0.0–1.0, Standard: 0.8

# Kontextfenster-Overrides pro Modell
model_settings:
  deepseek-chat:
    max_context_tokens: 64000

# Channel-Bridge — nicht-interaktive Alternative zu `claudy channel add`
channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0           # 0 = unbegrenzt
  stream_timeout_secs: 1800

# Agent-Overrides
agents:
  aider:
    binary: "aider"
    args: ["--message", "{prompt}"]
    timeout: 300
```

</details>

---

## Grundkonzepte

### Profil

Ein Startziel, das Anbieter-Metadaten und Authentifizierungsstrategie auflöst (eingebauter Anbieter, OpenRouter-Alias oder benutzerdefinierter Anbieter).

### Modus

Ein benanntes Claude-Konfigurationsverzeichnis unter `~/.claudy/modes/<name>/`.

Beim Ausführen von:

```bash
claudy <profile> <mode> [args...]
```

setzt Claudy:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

sodass Claude modusspezifische Konfigurationsdateien liest.

Modi eignen sich auch hervorragend für **dedizierte Claude-Frameworks und Toolkits**, die eine eigene `CLAUDE.md`, Skills, Agents oder Einstellungen mitliefern — wie [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code) oder ein beliebiges eigenes Harness. Anstatt die Standardkonfiguration zu verunreinigen, wird jedes Framework in seinem eigenen Modus isoliert:

```bash
# Einen dedizierten Modus für das Framework erstellen
claudy mode create gstack

# Die Framework-Konfiguration in das Modus-Verzeichnis kopieren oder verlinken
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Claude mit aktivem Framework starten
claudy <profile> gstack
```

Jedes Modus-Verzeichnis ist ein in sich geschlossenes `CLAUDE_CONFIG_DIR`, sodass Frameworks weder miteinander noch mit der Standardkonfiguration kollidieren.

<details>
<summary>Befehlsreferenz</summary>

## Befehlsreferenz

### Hauptbefehle

- `claudy ls` (Alias: `list`): konfigurierte/aufgelöste Profile auflisten.
- `claudy setup [provider]` (Alias: `config`): interaktive Anbieter-Einrichtung.
- `claudy show <profile>` (Alias: `info`): aufgelöste Anbieter-Details anzeigen.
- `claudy ping [profile]` (Alias: `test`): Anbieter-Konnektivität testen.
- `claudy doctor` (Alias: `status`): Version, Pfade und Profilanzahl anzeigen.
- `claudy sync` (Alias: `install`): Claudy-Binärdatei installieren/synchronisieren.
- `claudy update`: Claudy aktualisieren.
- `claudy uninstall`: installierte Dateien entfernen.
- `claudy mode <action> [name]`: Claude-Konfigurationsmodi verwalten.
- `claudy channel <subcommand>`: Channel-Bridge verwalten.
- `claudy mcp`: als MCP-Server für Agent-Bridge ausführen.
- `claudy analytics <subcommand>`: Nutzungsanalyse-Dashboard.

### Modusbefehle

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

Modusnamensregel: `[a-z0-9][a-z0-9_-]*` (`mode` ist reserviert).

### Channel-Befehle (optionale Bridge)

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

`channel add` führt durch Bot-Token, erlaubte Benutzer, Profil- und Moduszuordnung.

#### Unterstützte Plattformen

| Plattform | Erfassung | Interaktive Schaltflächen | Hinweise |
|----------|-----------|-------------------|-------|
| Telegram | Long-Polling + Webhook | Inline-Tastatur | Am vollständigsten |
| Slack | Event-Subscription-Webhook | Block-Kit-Aktionen | HMAC-SHA256-verifiziert |
| Discord | Interaktions-Webhook | Action-Row-Komponenten | Ed25519-verifiziert |

#### Channel-Bot-Befehle

Sobald der Bot läuft, antwortet er auf diese Befehle im Chat:

- `/help` — Verfügbare Befehle anzeigen
- `/cancel` — Aktuelle Aufgabe abbrechen
- `/model` — Claude-Modell ändern (interaktive Schaltflächen)
- `/yolo` — Auto-Allow-Berechtigungen umschalten
- `/status` — Sitzungsstatus, Profil, Modus, Git-Branch und Token-Verbrauch anzeigen
- `/sessions` — Letzte Claude-Sitzungen auflisten (mit Wechsel-Schaltflächen)
- `/projects` — Projekte auflisten (mit Navigations-Schaltflächen)
- `/new` — Neue Sitzung starten
- `/history` — Letzte Sitzungshistorie anzeigen

Senden Sie einen beliebigen anderen Text, um direkt mit Claude zu sprechen.

#### Berechtigungsabfragen

Wenn Claude die Genehmigung zur Nutzung eines Tools anfordert (Befehl ausführen, Datei bearbeiten usw.),
sendet der Bot eine interaktive Zulassen/Ablehnen-Abfrage in Ihren Chat. Durch Antippen einer Schaltfläche
wird die Antwort an Claude zurückgesendet und die Verarbeitung wird automatisch fortgesetzt.

#### Zugangsdaten

Channel-Zugangsdaten in `~/.claudy/secrets.env` speichern (siehe [Anbieter-Anmeldeinformationen](#anbieter-anmeldeinformationen-secretsenv) für das vollständige Format):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## Agent-MCP-Bridge

Führen Sie `claudy mcp` aus, um einen stdio-basierten MCP-Server zu starten, der es Claude Code ermöglicht, Aufgaben an andere lokal installierte KI-Coding-Agents zu delegieren.

```bash
claudy mcp run        # MCP-Server starten (wird von Claude Code aufgerufen)
claudy mcp install    # Claudy als MCP-Server in Claude Code-Einstellungen registrieren
claudy mcp uninstall  # Claudy aus Claude Code MCP-Einstellungen entfernen
```

`claudy mcp install` registriert sich automatisch in `~/.claude/settings.json`. Beim Erstellen eines Modus mit `claudy mode create <name>` wird auch in der Einstellungsdatei des Modus registriert. Keine manuelle Konfiguration erforderlich.

Zur manuellen Registrierung (oder in einer projekteigenen `.claude/settings.json`):

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

Claude Code sieht ein `ask_agent`-Tool, das alle installierten Agents verfügbar macht.

### Nutzungsbeispiel

Sobald registriert, kann Claude Code Aufgaben wie folgt delegieren:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code wählt den entsprechenden Agent aus, übergibt den Prompt und gibt das Ergebnis zurück. Sie können auch ein Arbeitsverzeichnis angeben:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### MCP-Registrierung überprüfen

```bash
# Prüfen, ob claudy registriert ist
cat ~/.claude/settings.json | grep -A3 claudy

# MCP-Server manuell testen
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### Unterstutzte Agents (automatisch aus PATH erkannt)

| Agent | Binärdatei | Headless-Befehl |
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

### Benutzerdefinierte Agents

Agents in `~/.claudy/config.yaml` unter dem Schlüssel `agents` hinzufügen (siehe [Konfiguration](#configyaml-schema) für das vollständige Schema):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

Ein Schlüssel, der mit einem eingebauten Agent übereinstimmt, überschreibt dessen Standardwerte. `{prompt}` in `args` wird durch die tatsächliche Aufgabe ersetzt.

## Nutzungsanalyse

> **Hinweis**: Die Analysefunktion befindet sich noch in der Entwicklung. Token-Zählungen, Kostenschätzungen und andere Metriken sind möglicherweise nicht vollständig genau. Verbesserungen in kommenden Versionen sind zu erwarten.

```bash
claudy analytics dashboard         # Lokales Analyse-Dashboard öffnen (Tauri 2)
claudy analytics ingest            # Sitzungsdaten aus ~/.claude/projects/ erfassen
claudy analytics ingest --full     # Alle Dateien neu erfassen (Prüfpunkte ignorieren)
claudy analytics ingest --project my-project  # Bestimmtes Projekt erfassen
claudy analytics recommend         # Nutzungsempfehlungen in der CLI anzeigen
claudy analytics export            # Analysedaten exportieren (JSON, Standard: 30 Tage)
claudy analytics export --format csv --days 7  # Als CSV für die letzten 7 Tage exportieren
claudy analytics sync-pricing      # Modellpreise von models.dev und Anthropic-Preisseite synchronisieren
claudy analytics recalculate       # Alle Kosten mit den neuesten Preisdaten neu berechnen
claudy analytics insights          # Kompakte JSON-Einblickzusammenfassung erstellen (Standard: 7 Tage)
claudy analytics insights --days 14  # Letzte 14 Tage analysieren
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # Spezifischer Zeitraum
claudy analytics insights --project my-project  # Nach Projekt filtern
```

### In Claude Code: `/analytics-insights`

Der schnellste Weg zur Analyse der Nutzung ist direkt in Claude Code. Der Skill `analytics-insights` ist automatisch verfügbar — einfach natürlich fragen:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude führt `claudy analytics insights` aus, analysiert das JSON und gibt einen strukturierten Bericht mit:

- **Kostentrends** — tägliche/wöchentliche Ausgaben mit Spike-Erkennung
- **Modellverteilung** — welche Modelle Sie nutzen und was sie pro Sitzung kosten
- **Tool-Muster** — meistgenutzte Tools, Fehlerraten, Effizienzbeobachtungen
- **Cache-Leistung** — Trefferquote und geschätzte Einsparungen
- **Handlungsrelevante Empfehlungen** — konkrete Vorschläge wie „einfache Aufgaben an turbo weiterleiten" mit geschätzter Dollar-Einsparung

Beispielausgabe (siehe [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json) für Rohdaten):

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

Keine manuellen Befehle, kein Kontextwechsel. Fragen Sie Claude nach Ihrer Nutzung und erhalten Sie sofort Antworten.

### Was die Analyse verfolgt

- **Tokens**: Detaillierte Trends von Eingabe-, Ausgabe- und Cache-Tokens der letzten 30 Tage, gruppiert nach Modell und Datum.
- **Tools**: Verteilungsanalyse, die zeigt, welche Tools Claude am häufigsten verwendet, einschließlich Aufrufanzahl, Fehlerraten und durchschnittlicher Ausführungszeit.
- **Kosten**: Echtzeit-Schätzung der Nutzungskosten basierend auf tatsächlichen Token-Preisen, einschließlich täglicher/wöchentlicher/monatlicher Prognosen und Trenderkennung (steigend/stabil/fallend).
- **Tipps (Empfehlungen)**: Datengestützte Optimierungsratschläge, wie z.B. Erkennung teurer Sitzungen, Vorschlag von Haiku für einfache Aufgaben und Identifizierung langer Konversationen, die von einer Kontextzusammenfassung profitieren könnten.
- **Projekte**: Automatische Zuordnung kryptischer Sitzungs-UUIDs zu lesbaren Projektordnernamen für besseren Kontext.

Daten werden in einer lokalen SQLite-Datenbank unter `~/.claudy/analytics/` gespeichert. Das Dashboard läuft als hochperformante lokale Tauri 2 + Svelte-App. Verwenden Sie die **[Sync]**-Schaltfläche im Dashboard, um Daten sofort aus Ihrem Claude CLI-Verlauf zu aktualisieren.

### Analyse-Dashboard
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="Analyse-Dashboard" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## Dateien und Verzeichnisstruktur

Standardmäßig speichert Claudy Daten unter:

```text
~/.claudy/
```

Wichtige Dateien/Verzeichnisse:

- `config.yaml`: Anbieter- + Channel- + Agent-Konfiguration.
- `secrets.env`: Anbieter-/Bot-Zugangsdaten.
- `launchers.json`: Launcher/Symlink-Manifest.
- `modes/`: Claude-Konfigurationsmodi.
- `session-patches/`: Sitzungs-Patch-Speicher.
- `channel/`: Channel-Laufzeitstatus (`pid`, Sitzungen, Audit-Log).
- `analytics/`: Analyse-SQLite-Datenbank und Prüfpunkte.
- `cache/update.json`: Update-Metadaten-Cache.

## Umgebungsvariablen

- `CLAUDY_HOME`: Claudy-Home-Verzeichnis überschreiben (Standard: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: wird von Claudy beim Start mit einem Modus automatisch gesetzt.

## Häufige Workflows

### Anbieter konfigurieren und starten

```bash
claudy setup
claudy <profile>
```

### Modus mit einem Anbieter verwenden

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` ist Claudys Abkürzung für `--dangerously-skip-permissions`.

### Dediziertes Claude-Framework in eigenem Modus ausführen

Frameworks wie gstack, superpowers oder ecc liefern eine eigene `CLAUDE.md`, Skills und Agents. Diese isoliert halten:

```bash
# Einmalige Einrichtung: Modus erstellen und mit Framework-Konfiguration befüllen
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Tägliche Nutzung: Claude mit aktivem Framework starten
claudy <profile> gstack
```

Zwischen Frameworks wechseln, ohne die Standardkonfiguration zu berühren:

```bash
claudy <profile> gstack      # gstack-Framework aktiv
claudy <profile> superpowers # superpowers-Framework aktiv
claudy <profile>             # Ihre Standardkonfiguration, unverändert
```

### Aufgaben über MCP an andere Agents delegieren

```bash
# 1) Sicherstellen, dass MCP registriert ist (passiert automatisch beim ersten `claudy mcp`)
claudy mcp

# 2) In Claude Code, um die Delegation an einen installierten Agent bitten:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Installations-/Konfigurationsstatus diagnostizieren

```bash
claudy doctor
claudy ping
```

## Fehlerbehebung

- **`profile not recognized`**: `claudy ls` ausführen und eine aufgelistete Profil-ID wählen.
- **`not configured`-Profil**: `claudy setup <provider>` ausführen, um Zugangsdaten hinzuzufügen.
- **Channel-Status fehlerhaft**: `claudy channel status` ausführen, dann mit `claudy channel stop` und `claudy channel start` neu starten.
- **Channel-Bot antwortet nicht**: `~/.claudy/channel/logs/server.log` auf Fehler prüfen. Bot-Token in `~/.claudy/secrets.env` verifizieren und sicherstellen, dass `allowed_users` Ihre Chat-Benutzer-ID enthält.
- **Berechtigungsabfrage erscheint nicht**: Sicherstellen, dass Claude CLI nicht mit `--dangerously-skip-permissions` ausgeführt wird. Die Abfrage wird nur ausgelöst, wenn Claude eine explizite Genehmigung für Tool-Nutzung benötigt.
- **Binärdatei nach Installation nicht gefunden**: Siehe den PATH-Hinweis im Abschnitt [Überprüfen](#überprüfen).
- **Agent wird in MCP nicht angezeigt**: Sicherstellen, dass sich die Agent-Binärdatei im `PATH` befindet (`which gemini`). Nur installierte Agents erscheinen in `tools/list`.
- **Agent-Timeout**: Timeout in `config.yaml` im Feld `agents` erhöhen (Standard: 120s).
- **MCP nicht registriert**: `claudy mcp` einmal manuell ausführen oder `~/.claude/settings.json` auf den Eintrag `mcpServers.claudy` prüfen.
- **Agent-Ausgabe abgeschnitten**: Agent-stdout ist auf 10 MB begrenzt. Bei großen Ausgaben den Agent anweisen, stattdessen in eine Datei zu schreiben.
- **Analyse-Daten fehlen**: `claudy analytics ingest` ausführen, um aus `~/.claude/projects/` zu befüllen. `--full` verwenden, um alles neu zu erfassen.

## Entwicklung

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Analyse-Backend testen (verwendet lokale DB)
cargo run --example test_dashboard --features analytics-ui

# Analyse-Dashboard starten (erfordert analytics-ui-Feature)
cargo run --features analytics-ui -- analytics dashboard
```

## Mitwirken

Beiträge sind willkommen! So kommen Sie gestartet:

1. Repository forken und einen Feature-Branch erstellen.
2. Änderungen mit entsprechenden Tests vornehmen.
3. Vor dem Einreichen `cargo test && cargo clippy -- -D warnings` ausführen.
4. Pull Request öffnen unter https://github.com/epicsagas/claudy.

Fehlerberichte und Feature-Anfragen sind willkommen über [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Danksagung

Dieses Projekt wurde inspiriert von [Clother](https://github.com/jolehuit/clother), einem Go-basierten Multi-Anbieter-Launcher für die Claude CLI. Claudy ist eine unabhängige Rust-Implementierung, von Grund auf neu gestaltet mit RAII-basierten Sitzungs-Guards, Signal-Weiterleitung, Launcher-Symlinks und tiefen Ökosystem-Integrationen einschließlich einer **voll ausgestatteten Channel-Bridge** (Telegram/Slack/Discord), der **Agent-MCP-Bridge** für cross-agent-Delegierung und eines **hochperformanten Analyse-Dashboards** basierend auf Tauri 2. Diese Erweiterungen spiegeln Claudys Entwicklung von einem einfachen Launcher zu einem umfassenden operativen Toolkit für Claude CLI-Benutzer wider.

## Lizenz

[Apache-2.0](../../LICENSE)
