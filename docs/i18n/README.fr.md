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

<p align="center"><b>Lanceur multi-fournisseur moderne pour Claude CLI.</b></p>

---

<p align="center">
Claudy vous permet d'exécuter Claude avec plusieurs providers via une interface de commandes unifiée, tout en maintenant les identifiants de chaque provider et les superpositions de configuration de Claude organisées dans un seul répertoire principal.
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../../assets/features-2048.png" alt="Pourquoi Claudy" width="100%" />

## Pourquoi Claudy

- **Lancement multi-fournisseur** : basculez entre le provider intégré, Z.AI, les alias OpenRouter, Ollama et les endpoints personnalisés compatibles Anthropic.
- **Config modes** : isolez la configuration de Claude (`CLAUDE.md`, `settings.json`, skills/plugins/agents) par Mode.
- **Résolution de Profile de provider** : unifie les providers intégrés, les providers personnalisés et les alias OpenRouter.
- **Comportement sécurisé du processus** : transfère SIGINT/SIGTERM au processus enfant de Claude.
- **UX opérationnel** : commandes d'installation/mise à jour/désinstallation, vérifications de statut et tests de connectivité.
- **Channel bridge optionnel** : exécutez un bot bridge local pour Telegram, Slack et Discord avec des invites de permissions interactives.
- **Agent MCP bridge** : déléguez des tâches depuis Claude Code vers d'autres agents IA locaux (Gemini, Codex, Aider, etc.) via MCP.
- **Analytics d'utilisation** : ingère les données de session depuis `~/.claude/projects/`, suit l'utilisation des tokens et les coûts par session/projet, et affiche un tableau de bord local avec des recommandations.

## Statut des Providers

> Claudy a été inspiré par [Clother](https://github.com/jolehuit/clother), un lanceur multi-fournisseur basé sur Go pour Claude CLI. **Seul le provider Z.AI a été entièrement testé**. Tous les autres providers alternatifs sont expérimentaux et non testés — utilisez-les à vos propres risques.

| Provider | Statut | Notes |
|---|---|---|
| Built-in (Anthropic) | ✅ Testé | Par défaut |
| Z.AI | ✅ Testé | Entièrement validé |
| OpenRouter alias | ⚠️ Expérimental | Non testé — utilisez à vos propres risques |
| Ollama | ⚠️ Expérimental | Non testé — utilisez à vos propres risques |
| Custom endpoint | ⚠️ Expérimental | Non testé — utilisez à vos propres risques |

## Prérequis

- macOS ou Linux
- Toolchain Rust (`cargo`) pour compiler/installer depuis le code source
- Claude CLI installé et disponible dans le `PATH`

## Installation

### Installer depuis crates.io

**Binaire pré-compilé (rapide, sans compilation)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**Toute plateforme — compiler depuis le code source**

```
cargo install claudy
```

**MacOS homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### Installer depuis le code source local

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### Vérifier

```bash
claudy --help
claudy --version
```

## Démarrage Rapide

```bash
# 1) Lister les profiles disponibles/résolus
claudy ls

# 2) Configurer les identifiants de manière interactive
claudy setup

# 3) Afficher les détails d'un profile
claudy show <profile>

# 4) Exécuter Claude avec un profile
claudy <profile> [claude-args...]
```

## Concepts Fondamentaux

### Profile

Une cible de lancement qui résout les métadonnées du provider et la stratégie d'authentification (provider intégré, alias OpenRouter ou provider personnalisé).

### Mode

Un répertoire de configuration de Claude nommé, situé dans `~/.claudy/modes/<name>/`.

Lorsque vous exécutez :

```bash
claudy <profile> <mode> [args...]
```

Claudy définit :

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

afin que Claude lise les fichiers de configuration spécifiques au Mode.

Les Modes sont également parfaitement adaptés pour exécuter des **frameworks et toolkits Claude dédiés** qui livrent leur propre `CLAUDE.md`, compétences, agents ou paramètres — comme [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code) ou tout harnais personnalisé. Plutôt que de polluer votre configuration par défaut, isolez chaque framework dans son propre Mode :

```bash
# Créer un Mode dédié pour le framework
claudy mode create gstack

# Copier ou lier symboliquement la config du framework dans le répertoire du Mode
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Lancer Claude avec ce framework actif
claudy <profile> gstack
```

Chaque répertoire Mode est un `CLAUDE_CONFIG_DIR` autonome, donc les frameworks ne se conflictuent jamais entre eux ni avec votre configuration par défaut.

## Référence des Commandes

### Commandes principales

- `claudy ls` (alias : `list`) : liste les profiles configurés/résolus.
- `claudy setup [provider]` (alias : `config`) : configuration interactive du provider.
- `claudy show <profile>` (alias : `info`) : affiche les détails résolus du provider.
- `claudy ping [profile]` (alias : `test`) : teste la connectivité du provider.
- `claudy doctor` (alias : `status`) : affiche la version, les chemins et le nombre de profiles.
- `claudy sync` (alias : `install`) : installe/synchronise le binaire claudy.
- `claudy update` : met à jour claudy.
- `claudy uninstall` : supprime les fichiers installés.
- `claudy mode <action> [name]` : gère les Config Modes de Claude.
- `claudy channel <subcommand>` : gère le Channel bridge.
- `claudy mcp` : exécute en tant que serveur MCP pour l'Agent bridge.
- `claudy analytics <subcommand>` : tableau de bord d'analytics d'utilisation.

### Commandes de Mode

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Règle de nommage du Mode : `[a-z0-9][a-z0-9_-]*` (`mode` est réservé).

### Commandes de Channel (bridge optionnel)

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

`channel add` vous guide à travers la configuration du token du bot, des utilisateurs autorisés, du profile et du mappage de Mode.

#### Plateformes prises en charge

| Plateforme | Ingestion | Boutons interactifs | Notes |
|----------|-----------|-------------------|-------|
| Telegram | Long-polling + webhook | Clavier inline | Le plus complet |
| Slack | Webhook d'abonnement aux événements | Actions Block Kit | Vérifié par HMAC-SHA256 |
| Discord | Webhook d'interaction | Composants Action row | Vérifié par Ed25519 |

#### Commandes du bot de Channel

Une fois en fonctionnement, le bot répond à ces commandes dans le chat :

- `/help` — Affiche les commandes disponibles
- `/cancel` — Annule la tâche en cours
- `/model` — Change le modèle Claude (boutons interactifs)
- `/yolo` — Active/désactive l'auto-approbation des permissions
- `/status` — Affiche le statut de la session, le profile, le Mode, la branche git et l'utilisation des tokens
- `/sessions` — Liste les sessions Claude récentes (avec boutons de changement)
- `/projects` — Liste les projets (avec boutons de navigation)
- `/new` — Démarre une nouvelle session
- `/history` — Affiche l'historique des sessions récentes

Envoyez n'importe quel autre texte pour parler directement à Claude.

#### Invites de permissions

Lorsque Claude demande l'approbation pour utiliser un outil (exécuter une commande, modifier un fichier, etc.), le bot envoie une invite interactive Autoriser/Refuser dans votre chat. Appuyer sur un bouton renvoie la réponse à Claude et le traitement se poursuit automatiquement.

#### Secrets

Stockez les identifiants dans `~/.claudy/secrets.env` :

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP bridge

Exécutez `claudy mcp` pour démarrer un serveur MCP basé sur stdio qui permet à Claude Code de déléguer des tâches à d'autres agents IA installés localement.

```bash
claudy mcp
```

Au premier lancement, claudy s'enregistre automatiquement dans `~/.claude/settings.json`. Lorsque vous créez un Mode avec `claudy mode create <name>`, il s'enregistre également dans le fichier de configuration du Mode. Aucune configuration manuelle n'est nécessaire.

Pour enregistrer manuellement (ou dans un `.claude/settings.json` au niveau du projet) :

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

Claude Code verra un outil `ask_agent` qui expose tous les agents installés.

#### Exemple d'utilisation

Une fois enregistré, Claude Code peut déléguer des tâches de cette façon :

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code sélectionne l'agent approprié, transmet le prompt et retourne le résultat. Vous pouvez également spécifier un répertoire de travail :

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### Vérifier l'enregistrement MCP

```bash
# Vérifier si claudy est enregistré
cat ~/.claude/settings.json | grep -A3 claudy

# Tester le serveur MCP manuellement
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### Agents pris en charge (détectés automatiquement depuis PATH)

| Agent | Binaire | Commande headless |
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

#### Agents personnalisés

Ajoutez des agents dans `~/.claudy/config.yaml` :

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

La même clé qu'un agent intégré remplace ses valeurs par défaut. `{prompt}` dans `args` est remplacé par la tâche réelle.

### Commandes Analytics

> **Note** : La fonctionnalité analytics est encore en cours de développement. Les comptages de tokens, les estimations de coûts et d'autres métriques peuvent ne pas être entièrement précis. Des améliorations sont prévues dans les prochaines versions.

```bash
claudy analytics dashboard         # Ouvrir le tableau de bord analytics local (Tauri 2)
claudy analytics ingest            # Ingérer les données de session depuis ~/.claude/projects/
claudy analytics ingest --full     # Réingérer tous les fichiers (ignorer les checkpoints)
claudy analytics ingest --project my-project  # Ingérer un projet spécifique
claudy analytics recommend         # Afficher les recommandations d'utilisation dans le CLI
claudy analytics export            # Exporter les données analytics (JSON, 30 jours par défaut)
claudy analytics export --format csv --days 7  # Exporter en CSV pour les 7 derniers jours
```

Analytics suit :

- **Tokens** : Tendances détaillées des tokens d'entrée, de sortie et de cache sur les 30 derniers jours, regroupés par modèle et par date.
- **Tools** : Analyse de distribution montrant quels outils Claude utilise le plus fréquemment, y compris les comptages d'appels, les taux d'erreur et le temps d'exécution moyen.
- **Coût** : Estimation en temps réel des coûts d'utilisation basée sur les prix réels des tokens, incluant des prévisions quotidiennes/hebdomadaires/mensuelles et la détection de tendances (croissante/stable/décroissante).
- **Conseils (Recommandations)** : Conseils d'optimisation basés sur les données, comme la détection des sessions à coût élevé, la suggestion de Haiku pour les tâches simples et l'identification des longues conversations pouvant bénéficier d'une résumisation du contexte.
- **Projets** : Mappe automatiquement les UUIDs cryptiques de session vers des noms de dossiers de projets lisibles par les humains pour un meilleur contexte.

Les données sont stockées dans une base de données SQLite locale sous `~/.claudy/analytics/`. Le tableau de bord s'exécute comme une application locale haute performance avec Tauri 2 + Svelte. Utilisez le bouton **[Sync]** dans le tableau de bord pour actualiser instantanément les données depuis votre historique Claude CLI.

<img src="../../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## Fichiers et Structure des Répertoires

Par défaut, Claudy stocke les données dans :

```text
~/.claudy/
```

Fichiers/répertoires importants :

- `config.yaml` : configuration du provider, du Channel et de l'agent.
- `secrets.env` : identifiants du provider/bot.
- `launchers.json` : manifeste des lanceurs/symlinks.
- `modes/` : Config Modes de Claude.
- `session-patches/` : stockage des patches de session.
- `channel/` : état d'exécution du Channel (`pid`, sessions, journal d'audit).
- `analytics/` : base de données SQLite et checkpoints d'analytics.
- `cache/update.json` : cache des métadonnées de mise à jour.

## Variables d'Environnement

- `CLAUDY_HOME` : remplace le répertoire principal de Claudy (par défaut : `~/.claudy`).
- `CLAUDE_CONFIG_DIR` : défini automatiquement par Claudy lors du lancement avec un Mode.

## Flux de Travail Courants

### Configurer et lancer un provider

```bash
claudy setup
claudy <profile>
```

### Utiliser un Mode avec un provider

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` est le raccourci de claudy pour `--dangerously-skip-permissions`.

### Exécuter un framework Claude dédié dans son propre Mode

Les frameworks comme gstack, superpowers ou ecc fournissent leur propre `CLAUDE.md`, compétences et agents. Gardez-les isolés :

```bash
# Configuration unique : créer le Mode et y intégrer la config du framework
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Utilisation quotidienne : lancer Claude avec le framework actif
claudy <profile> gstack
```

Basculer entre frameworks sans toucher à la configuration par défaut :

```bash
claudy <profile> gstack      # framework gstack actif
claudy <profile> superpowers # framework superpowers actif
claudy <profile>             # configuration par défaut, inchangée
```

### Déléguer des tâches à d'autres agents via MCP

```bash
# 1) S'assurer que MCP est enregistré (se produit automatiquement au premier `claudy mcp`)
claudy mcp

# 2) Dans Claude Code, lui demander de déléguer à n'importe quel agent installé :
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnostiquer l'état d'installation/configuration

```bash
claudy doctor
claudy ping
```

## Résolution des Problèmes

- **`profile not recognized`** : exécutez `claudy ls` et choisissez un ID de profile dans la liste.
- **Profile `not configured`** : exécutez `claudy setup <provider>` pour ajouter des identifiants.
- **Statut du Channel défaillant** : exécutez `claudy channel status`, puis redémarrez avec `claudy channel stop` et `claudy channel start`.
- **Bot du Channel ne répond pas** : vérifiez `~/.claudy/channel/logs/server.log` pour les erreurs. Vérifiez le token du bot dans `~/.claudy/secrets.env` et que `allowed_users` inclut votre ID d'utilisateur de chat.
- **L'invite de permission n'apparaît pas** : assurez-vous que Claude CLI n'est pas exécuté avec `--dangerously-skip-permissions`. L'invite ne se déclenche que lorsque Claude a besoin d'une approbation explicite pour l'utilisation d'outils.
- **Binaire introuvable après l'installation** : assurez-vous que le répertoire bin de Claudy est dans le `PATH`, puis redémarrez votre shell.
- **Agent non visible dans MCP** : assurez-vous que le binaire de l'agent est dans le `PATH` (`which gemini`). Seuls les agents installés apparaissent dans `tools/list`.
- **Timeout de l'agent** : augmentez le timeout dans le champ agents de `config.yaml` (par défaut : 120s).
- **MCP non enregistré** : exécutez `claudy mcp` une fois manuellement, ou vérifiez `~/.claude/settings.json` pour l'entrée `mcpServers.claudy`.
- **Sortie de l'agent tronquée** : la sortie stdout de l'agent est limitée à 10 Mo. Pour les sorties volumineuses, redirigez l'agent pour qu'il écrive dans un fichier.
- **Données analytics manquantes** : exécutez `claudy analytics ingest` pour alimenter depuis `~/.claude/projects/`. Utilisez `--full` pour tout réingérer.

## Développement

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Tester le backend analytics (utilise la BD locale)
cargo run --example test_dashboard --features analytics-ui

# Lancer le tableau de bord analytics (nécessite la feature analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## Contribuer

Les contributions sont les bienvenues ! Voici comment commencer :

1. Forkez le dépôt et créez une branche de fonctionnalité.
2. Effectuez vos modifications avec des tests le cas échéant.
3. Exécutez `cargo test && cargo clippy -- -D warnings` avant de soumettre.
4. Ouvrez une Pull Request sur https://github.com/epicsagas/claudy.

Les rapports de bugs et les demandes de fonctionnalités sont les bienvenus via [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Remerciements

Ce projet a été inspiré par [Clother](https://github.com/jolehuit/clother), un lanceur multi-fournisseur basé sur Go pour Claude CLI. Claudy est une implémentation Rust indépendante, repensée de zéro avec des gardes de session basées sur RAII, le transfert de signaux, des symlinks de lanceurs et des intégrations profondes avec l'écosystème, notamment un **Channel Bridge complet** (Telegram/Slack/Discord), l'**Agent MCP Bridge** pour la délégation inter-agents, et un **tableau de bord Analytics haute performance** construit avec Tauri 2. Ces ajouts reflètent la transition de Claudy d'un simple lanceur vers une boîte à outils opérationnelle complète pour les utilisateurs de Claude CLI.

## Licence

[Apache-2.0](../../LICENSE)
