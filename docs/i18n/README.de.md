[← English](../../README.md)

<h1 align="center">claudy</h1>

<p align="center"><b>Ein Befehl. Jeder Provider. Volle Kontrolle über das Claude CLI.</b></p>

---

<p align="center">
Schluss mit dem Jonglieren von Umgebungsvariablen und Konfigurationsdateien.<br/>
Mit Claudy wechseln Sie mit einem einzigen Befehl zwischen Anthropic, Z.AI, OpenRouter, Ollama und benutzerdefinierten Endpoints — Zugangsdaten, Konfigurationsmodi und Claude-Frameworks bleiben sauber pro Profil isoliert.
</p>

<p align="center">
<b>Multi-Provider · Konfigurations-Isolation · Channel-Bridge · Lokale Agent-Bridge · Nutzungsanalyse</b>
</p>

---

<p align="center"><b>Moderner Multi-Provider-Launcher für das Claude CLI.</b></p>

---

<p align="center">
Claudy ermöglicht es Ihnen, Claude mit mehreren Providern über eine einheitliche Befehlsoberfläche auszuführen, während die Provider-Anmeldeinformationen und Claude-Konfigurations-Overlays in einem einzigen Hauptverzeichnis organisiert bleiben.
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../assets/features-2048.png" alt="Warum Claudy" width="100%" />

## Warum Claudy

- **Multi-Provider-Launch**: Wechseln Sie zwischen integriertem Provider, Z.AI, OpenRouter-Alias, Ollama und benutzerdefinierten Anthropic-kompatiblen Endpunkten.
- **Config Modes**: Isolieren Sie die Claude-Konfiguration (`CLAUDE.md`, `settings.json`, Skills/Plugins/Agents) pro Mode.
- **Provider-Profile-Auflösung**: Vereinheitlicht integrierte Provider, benutzerdefinierte Provider und OpenRouter-Aliases.
- **Sicheres Prozessverhalten**: Leitet SIGINT/SIGTERM an den untergeordneten Claude-Prozess weiter.
- **Operationelle UX**: Installations-/Update-/Deinstallationsbefehle, Statusprüfungen und Konnektivitätstests.
- **Optionale Channel Bridge**: Führen Sie eine lokale Bot-Bridge für Telegram, Slack und Discord mit interaktiven Berechtigungsabfragen aus.
- **Agent MCP Bridge**: Delegieren Sie Aufgaben von Claude Code an andere lokale KI-Agenten (Gemini, Codex, Aider usw.) über MCP.
- **Nutzungsanalysen**: Liest Sitzungsdaten aus `~/.claude/projects/`, verfolgt Token-Nutzung und Kosten pro Sitzung/Projekt und zeigt ein lokales Dashboard mit Empfehlungen an.

## Unterstützte Provider

> Claudy wurde von [Clother](https://github.com/jolehuit/clother) inspiriert, einem Go-basierten Multi-Provider-Launcher für das Claude CLI. Z.AI wurde am gründlichsten getestet. Wenn Sie bei anderen Providern auf Probleme stoßen, [öffnen Sie bitte ein Issue](https://github.com/epicsagas/claudy/issues).

| Provider | Status | Hinweise |
|---|---|---|
| Built-in (Anthropic) | ✅ Getestet | Standard |
| Z.AI | ✅ Getestet | |
| OpenRouter alias | ⚠️ Experimentell | Noch nicht vollständig getestet — bitte auf GitHub melden |
| Ollama | ⚠️ Experimentell | Noch nicht vollständig getestet — bitte auf GitHub melden |
| Custom endpoint | ⚠️ Experimentell | Noch nicht vollständig getestet — bitte auf GitHub melden |

## Voraussetzungen

- macOS oder Linux
- Rust-Toolchain (`cargo`) für die Kompilierung/Installation aus dem Quellcode
- Claude CLI installiert und im `PATH` verfügbar

## Installation

### Installation über crates.io

**Vorkompiliertes Binär (schnell, keine Kompilierung)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**Jede Plattform — aus dem Quellcode kompilieren**

```
cargo install claudy
```

**MacOS homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### Aus lokalem Quellcode installieren

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### Überprüfen

```bash
claudy --help
claudy --version
```

## Schnellstart

<img src="docs/assets/demo.gif" alt="Quick Start" width="100%" />

```bash
# 1) Verfügbare/aufgelöste Profiles auflisten
claudy ls

# 2) Anmeldeinformationen interaktiv konfigurieren
claudy setup

# 3) Details eines Profiles anzeigen
claudy show <profile>

# 4) Claude mit einem Profile ausführen
claudy <profile> [claude-args...]
```

## Grundlegende Konzepte

### Profile

Ein Startziel, das Provider-Metadaten und die Authentifizierungsstrategie auflöst (integrierter Provider, OpenRouter-Alias oder benutzerdefinierter Provider).

### Mode

Ein benanntes Claude-Konfigurationsverzeichnis unter `~/.claudy/modes/<name>/`.

Wenn Sie ausführen:

```bash
claudy <profile> <mode> [args...]
```

setzt Claudy:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

damit Claude Mode-spezifische Konfigurationsdateien liest.

Modes eignen sich auch ideal für **dedizierte Claude-Frameworks und Toolkits**, die eigene `CLAUDE.md`-, Skills-, Agenten- oder Settings-Dateien mitbringen — wie [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code) oder eigene Harnesses. Statt die Standard-Konfiguration zu verschmutzen, kann jedes Framework in einem eigenen Mode isoliert werden:

```bash
# Dedizierten Mode für das Framework erstellen
claudy mode create gstack

# Framework-Konfiguration in das Mode-Verzeichnis kopieren oder verlinken
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Claude mit diesem Framework starten
claudy <profile> gstack
```

Jedes Mode-Verzeichnis ist ein eigenständiges `CLAUDE_CONFIG_DIR`, sodass Frameworks sich gegenseitig und die Standard-Konfiguration nicht beeinflussen.

## Befehlsreferenz

### Hauptbefehle

- `claudy ls` (Alias: `list`): listet konfigurierte/aufgelöste Profiles auf.
- `claudy setup [provider]` (Alias: `config`): interaktive Provider-Einrichtung.
- `claudy show <profile>` (Alias: `info`): zeigt aufgelöste Provider-Details an.
- `claudy ping [profile]` (Alias: `test`): testet die Provider-Konnektivität.
- `claudy doctor` (Alias: `status`): zeigt Version, Pfade und Profile-Anzahl an.
- `claudy sync` (Alias: `install`): installiert/synchronisiert das claudy-Binär.
- `claudy update`: aktualisiert claudy.
- `claudy uninstall`: entfernt installierte Dateien.
- `claudy mode <action> [name]`: verwaltet Claude Config Modes.
- `claudy channel <subcommand>`: verwaltet die Channel Bridge.
- `claudy mcp`: läuft als MCP-Server für die Agent Bridge.
- `claudy analytics <subcommand>`: Nutzungsanalysen-Dashboard.

### Mode-Befehle

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Benennungsregel für Mode: `[a-z0-9][a-z0-9_-]*` (`mode` ist reserviert).

### Channel-Befehle (optionale Bridge)

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

`channel add` führt Sie durch Bot-Token, erlaubte Benutzer, Profile und Mode-Zuordnung.

#### Unterstützte Plattformen

| Plattform | Aufnahme | Interaktive Schaltflächen | Hinweise |
|----------|-----------|-------------------|-------|
| Telegram | Long-Polling + Webhook | Inline-Tastatur | Am vollständigsten |
| Slack | Event-Abonnement-Webhook | Block Kit-Aktionen | HMAC-SHA256 verifiziert |
| Discord | Interaktions-Webhook | Action-Row-Komponenten | Ed25519 verifiziert |

#### Bot-Befehle für Channel

Sobald der Bot läuft, reagiert er auf diese Befehle im Chat:

- `/help` — Verfügbare Befehle anzeigen
- `/cancel` — Aktuelle Aufgabe abbrechen
- `/model` — Claude-Modell wechseln (interaktive Schaltflächen)
- `/yolo` — Auto-Erlauben von Berechtigungen umschalten
- `/status` — Sitzungsstatus, Profile, Mode, Git-Branch und Token-Nutzung anzeigen
- `/sessions` — Aktuelle Claude-Sitzungen auflisten (mit Wechselschaltflächen)
- `/projects` — Projekte auflisten (mit Navigationsschaltflächen)
- `/new` — Neue Sitzung starten
- `/history` — Verlauf der letzten Sitzungen anzeigen

Senden Sie beliebigen anderen Text, um direkt mit Claude zu sprechen.

#### Berechtigungsabfragen

Wenn Claude die Genehmigung zur Verwendung eines Tools anfordert (einen Befehl ausführen, eine Datei bearbeiten usw.), sendet der Bot eine interaktive Erlauben/Ablehnen-Anfrage in Ihren Chat. Durch Tippen auf eine Schaltfläche wird die Antwort an Claude zurückgesendet und die Verarbeitung wird automatisch fortgesetzt.

#### Geheimnisse

Speichern Sie Anmeldeinformationen in `~/.claudy/secrets.env`:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP Bridge

Führen Sie `claudy mcp` aus, um einen stdio-basierten MCP-Server zu starten, der es Claude Code ermöglicht, Aufgaben an andere lokal installierte KI-Coding-Agenten zu delegieren.

```bash
claudy mcp
```

Beim ersten Ausführen registriert sich claudy automatisch in `~/.claude/settings.json`. Wenn Sie einen Mode mit `claudy mode create <name>` erstellen, registriert er sich auch in der Einstellungsdatei des Modes. Keine manuelle Konfiguration erforderlich.

Für die manuelle Registrierung (oder in einer projektbezogenen `.claude/settings.json`):

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

Claude Code sieht ein `ask_agent`-Tool, das alle installierten Agenten bereitstellt.

#### Verwendungsbeispiel

Nach der Registrierung kann Claude Code Aufgaben folgendermaßen delegieren:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code wählt den geeigneten Agenten aus, übergibt den Prompt und gibt das Ergebnis zurück. Sie können auch ein Arbeitsverzeichnis angeben:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### MCP-Registrierung überprüfen

```bash
# Prüfen ob claudy registriert ist
cat ~/.claude/settings.json | grep -A3 claudy

# MCP-Server manuell testen
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### Unterstützte Agenten (automatisch aus PATH erkannt)

| Agent | Binär | Headless-Befehl |
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

#### Benutzerdefinierte Agenten

Fügen Sie Agenten in `~/.claudy/config.yaml` hinzu:

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

Derselbe Schlüssel wie ein integrierter Agent überschreibt dessen Standardwerte. `{prompt}` in `args` wird durch die eigentliche Aufgabe ersetzt.

### Analytics-Befehle

> **Hinweis**: Die Analytics-Funktion befindet sich noch in der Entwicklung. Token-Zählungen, Kostenschätzungen und andere Metriken sind möglicherweise nicht vollständig genau. Verbesserungen sind in kommenden Versionen zu erwarten.

```bash
claudy analytics dashboard         # Lokales Analytics-Dashboard öffnen (Tauri 2)
claudy analytics ingest            # Sitzungsdaten aus ~/.claude/projects/ einlesen
claudy analytics ingest --full     # Alle Dateien neu einlesen (Checkpoints ignorieren)
claudy analytics ingest --project my-project  # Bestimmtes Projekt einlesen
claudy analytics recommend         # Nutzungsempfehlungen im CLI anzeigen
claudy analytics export            # Analytics-Daten exportieren (JSON, Standard 30 Tage)
claudy analytics export --format csv --days 7  # Als CSV für die letzten 7 Tage exportieren
```

Analytics verfolgt:

- **Tokens**: Detaillierte Trends von Eingabe-, Ausgabe- und Cache-Tokens über die letzten 30 Tage, gruppiert nach Modell und Datum.
- **Tools**: Verteilungsanalyse, die zeigt, welche Tools Claude am häufigsten verwendet, einschließlich Aufrufzählungen, Fehlerquoten und durchschnittlicher Ausführungszeit.
- **Kosten**: Echtzeit-Schätzung der Nutzungskosten basierend auf tatsächlichen Token-Preisen, einschließlich täglicher/wöchentlicher/monatlicher Prognosen und Trendeerkennung (steigend/stabil/fallend).
- **Tipps (Empfehlungen)**: Datengestützte Optimierungshinweise, wie die Erkennung von kostenintensiven Sitzungen, Empfehlung von Haiku für einfache Aufgaben und Identifizierung langer Gespräche, die von einer Kontextzusammenfassung profitieren könnten.
- **Projekte**: Ordnet kryptische Sitzungs-UUIDs automatisch lesbaren Projektordnernamen zu für besseren Kontext.

Daten werden in einer lokalen SQLite-Datenbank unter `~/.claudy/analytics/` gespeichert. Das Dashboard läuft als hochperformante lokale Tauri 2 + Svelte-App. Verwenden Sie die **[Sync]**-Schaltfläche im Dashboard, um Daten aus Ihrem Claude CLI-Verlauf sofort zu aktualisieren.

<img src="../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## Dateien und Verzeichnisstruktur

Standardmäßig speichert Claudy Daten unter:

```text
~/.claudy/
```

Wichtige Dateien/Verzeichnisse:

- `config.yaml`: Provider-, Channel- und Agenten-Konfiguration.
- `secrets.env`: Provider/Bot-Anmeldeinformationen.
- `launchers.json`: Launcher/Symlink-Manifest.
- `modes/`: Claude Config Modes.
- `session-patches/`: Sitzungs-Patch-Speicher.
- `channel/`: Channel-Laufzeitstatus (`pid`, Sitzungen, Audit-Log).
- `analytics/`: Analytics-SQLite-Datenbank und Checkpoints.
- `cache/update.json`: Update-Metadaten-Cache.

## Umgebungsvariablen

- `CLAUDY_HOME`: überschreibt das Claudy-Hauptverzeichnis (Standard: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: wird von Claudy beim Start mit einem Mode automatisch gesetzt.

## Häufige Arbeitsabläufe

### Einen Provider konfigurieren und starten

```bash
claudy setup
claudy <profile>
```

### Einen Mode mit einem Provider verwenden

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` ist Claudys Kurzform für `--dangerously-skip-permissions`.

### Dediziertes Claude-Framework in einem eigenen Mode ausführen

Frameworks wie gstack, superpowers oder ecc bringen eigene `CLAUDE.md`-, Skills- und Agenten-Dateien mit. Halten Sie sie isoliert:

```bash
# Einmalige Einrichtung: Mode erstellen und Framework-Konfiguration einbinden
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Tägliche Nutzung: Claude mit aktivem Framework starten
claudy <profile> gstack
```

Zwischen Frameworks wechseln, ohne die Standard-Konfiguration anzufassen:

```bash
claudy <profile> gstack      # gstack-Framework aktiv
claudy <profile> superpowers # superpowers-Framework aktiv
claudy <profile>             # Standard-Konfiguration, unverändert
```

### Aufgaben über MCP an andere Agenten delegieren

```bash
# 1) Sicherstellen, dass MCP registriert ist (geschieht automatisch beim ersten `claudy mcp`)
claudy mcp

# 2) In Claude Code darum bitten, an einen installierten Agenten zu delegieren:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Installations-/Konfigurationsstatus diagnostizieren

```bash
claudy doctor
claudy ping
```

## Fehlerbehebung

- **`profile not recognized`**: Führen Sie `claudy ls` aus und wählen Sie eine aufgelistete Profile-ID.
- **Profile `not configured`**: Führen Sie `claudy setup <provider>` aus, um Anmeldeinformationen hinzuzufügen.
- **Channel-Status fehlerhaft**: Führen Sie `claudy channel status` aus und starten Sie dann mit `claudy channel stop` und `claudy channel start` neu.
- **Channel-Bot antwortet nicht**: Überprüfen Sie `~/.claudy/channel/logs/server.log` auf Fehler. Überprüfen Sie das Bot-Token in `~/.claudy/secrets.env` und ob `allowed_users` Ihre Chat-Benutzer-ID enthält.
- **Berechtigungsabfrage erscheint nicht**: Stellen Sie sicher, dass Claude CLI nicht mit `--dangerously-skip-permissions` ausgeführt wird. Die Abfrage wird nur ausgelöst, wenn Claude eine explizite Genehmigung für die Tool-Nutzung benötigt.
- **Binär nach der Installation nicht gefunden**: Stellen Sie sicher, dass das bin-Verzeichnis von Claudy im `PATH` ist, und starten Sie dann Ihre Shell neu.
- **Agent erscheint nicht in MCP**: Stellen Sie sicher, dass das Agent-Binär im `PATH` ist (`which gemini`). Nur installierte Agenten erscheinen in `tools/list`.
- **Agent-Timeout**: Erhöhen Sie das Timeout im Agenten-Feld von `config.yaml` (Standard: 120s).
- **MCP nicht registriert**: Führen Sie `claudy mcp` einmal manuell aus, oder überprüfen Sie `~/.claude/settings.json` auf den Eintrag `mcpServers.claudy`.
- **Agentenausgabe abgeschnitten**: Die stdout-Ausgabe des Agenten ist auf 10 MB begrenzt. Bei großen Ausgaben leiten Sie den Agenten um, in eine Datei zu schreiben.
- **Analytics-Daten fehlen**: Führen Sie `claudy analytics ingest` aus, um aus `~/.claude/projects/` zu befüllen. Verwenden Sie `--full`, um alles neu einzulesen.

## Entwicklung

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Analytics-Backend testen (verwendet lokale DB)
cargo run --example test_dashboard --features analytics-ui

# Analytics-Dashboard starten (erfordert das analytics-ui-Feature)
cargo run --features analytics-ui -- analytics dashboard
```

## Mitwirken

Beiträge sind willkommen! So können Sie loslegen:

1. Forken Sie das Repository und erstellen Sie einen Feature-Branch.
2. Nehmen Sie Ihre Änderungen mit Tests vor, wo angemessen.
3. Führen Sie `cargo test && cargo clippy -- -D warnings` vor dem Einreichen aus.
4. Öffnen Sie einen Pull Request unter https://github.com/epicsagas/claudy.

Fehlerberichte und Feature-Anfragen sind über [GitHub Issues](https://github.com/epicsagas/claudy/issues) willkommen.

## Danksagungen

Dieses Projekt wurde von [Clother](https://github.com/jolehuit/clother) inspiriert, einem Go-basierten Multi-Provider-Launcher für das Claude CLI. Claudy ist eine unabhängige Rust-Implementierung, von Grund auf neu gestaltet mit RAII-basierten Sitzungswächtern, Signal-Weiterleitung, Launcher-Symlinks und tiefen Ökosystem-Integrationen, einschließlich einer **vollständigen Channel Bridge** (Telegram/Slack/Discord), der **Agent MCP Bridge** für agentenübergreifende Delegation und einem **hochperformanten Analytics-Dashboard** auf Basis von Tauri 2. Diese Ergänzungen spiegeln Claudys Übergang von einem einfachen Launcher zu einem umfassenden operativen Toolkit für Claude CLI-Nutzer wider.

## Lizenz

[Apache-2.0](../../LICENSE)
