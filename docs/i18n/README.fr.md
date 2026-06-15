<h1 align="center">claudy</h1>

<p align="center"><b>Une seule commande. Tous les fournisseurs. Contrôle total sur Claude CLI.</b></p>

<p align="center">
Fini le jonglage avec les variables d'environnement et les fichiers de configuration.<br/>
Claudy vous permet de basculer entre Anthropic, Z.AI, OpenRouter, Ollama et des points de terminaison personnalisés en une seule commande — en gardant les identifiants, les modes de configuration et les frameworks Claude proprement isolés par profil.
</p>

<p align="center">
<b>Multi-fournisseurs · Isolation de configuration · Pont de canaux · Pont d'agents local · Analytique d'utilisation</b>
</p>

---

<p align="center">
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
  <img alt="Pourquoi Claudy" src="../assets/features-2048.png" width="100%">
</picture>

## Pourquoi Claudy

| | Fonctionnalite | Pourquoi c'est important |
|--|---------|----------------|
| 🔄 | Lancement multi-fournisseurs | Basculez entre Anthropic, Z.AI, OpenRouter, Ollama et des points de terminaison personnalises en une seule commande |
| 📦 | Modes de configuration | Isolez `CLAUDE.md`, les paramètres, les compétences et les agents par mode — aucune contamination croisée |
| 🔗 | Pont MCP d'agents | Déléguez des tâches de Claude Code à agy, Codex, Aider et plus de 20 autres agents |
| 💬 | Pont de canaux | Exécutez des bots Telegram, Slack et Discord avec des invites de permission interactives |
| 📊 | Analytique d'utilisation | Suivez l'utilisation des jetons, les coûts et les modèles d'outils avec un tableau de bord Tauri local |
| 🔐 | Contrôle de processus sûr | Transmission SIGINT/SIGTERM, écritures atomiques de configuration, stockage des identifiants en mode 0600 |
| 🔀 | Continuité de session inter-fournisseurs | Réparer automatiquement les sessions Z.AI/GLM pour les reprendre sans interruption avec l'API Anthropic |
| 🛠️ | UX opérationnelle | Installation, mise à jour, désinstallation, diagnostic, ping — tout depuis un seul binaire |

## Fournisseurs pris en charge

> Claudy a été inspiré par [Clother](https://github.com/jolehuit/clother), un lanceur multi-fournisseurs basé sur Go pour Claude CLI. Z.AI est le fournisseur le plus testé. Si vous rencontrez des problèmes avec d'autres fournisseurs, veuillez [ouvrir un ticket](https://github.com/epicsagas/claudy/issues).

| Fournisseur | Statut | Remarques |
|---|---|---|
| Intégré (Anthropic) | ✅ Teste | Par défaut |
| Z.AI | ✅ Teste | |
| Alias OpenRouter | ⚠️ Expérimental | Non entièrement testé — signalez les problèmes sur GitHub |
| Ollama | ⚠️ Expérimental | Non entièrement testé — signalez les problèmes sur GitHub |
| Point de terminaison personnalisé | ⚠️ Expérimental | Non entièrement testé — signalez les problèmes sur GitHub |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="démo" src="../assets/demo.gif" width="100%">
</picture>

## Démarrage rapide

**1. Installer**

macOS / Linux :

```bash
brew install epicsagas/tap/claudy
```

Pas de Homebrew ? Utilisez le script d'installation :

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows :

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Via la chaîne d'outils Rust :

```bash
cargo binstall claudy   # binaire pré-compilé (rapide)
cargo install claudy    # compiler depuis les sources
```

**2. Configurer**

```bash
claudy install                        # initialiser les répertoires, la configuration, les secrets
echo 'ANTHROPIC_API_KEY=votre-clé' >> ~/.claudy/secrets.env
```

**3. Lancer**

```bash
claudy                                # fournisseur par défaut
claudy zai                            # fournisseur Z.AI
claudy openrouter sonnet              # alias OpenRouter
```

**4. Mettre à jour**

```bash
brew upgrade claudy          # Homebrew
claudy update                # mise à jour intégrée
# ou ré-exécuter le script d'installation / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>Identifiants des fournisseurs</summary>

| Variable | Fournisseur |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (natif) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI Chine |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax Chine |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `ALIBABA_API_KEY` | Alibaba Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (tous les alias) |

Les fournisseurs personnalisés utilisent la variable `api_key_env` définie dans leur entrée `custom_providers`.

</details>

<details>
<summary>Schéma config.yaml</summary>

Toute la configuration se trouve dans `~/.claudy/config.yaml`. Ajoutez uniquement les sections dont vous avez besoin — les valeurs par défaut sont utilisées pour tout ce qui est omis.

> Référence complète : [docs/config.md](../config.md)

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

## Concepts clés

### Profil

Une cible de lancement qui résout les métadonnées du fournisseur et la stratégie d'authentification (fournisseur intégré, alias OpenRouter ou fournisseur personnalisé).

### Mode

Un répertoire de configuration Claude nommé situé dans `~/.claudy/modes/<nom>/`.

Lorsque vous exécutez :

```bash
claudy <profil> <mode> [args...]
```

Claudy définit :

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

pour que Claude lise les fichiers de configuration spécifiques au mode.

Les modes sont également parfaitement adaptés aux **frameworks et boîtes à outils Claude dédiés** qui fournissent leur propre `CLAUDE.md`, compétences, agents ou paramètres — tels que [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code), notre propre [epic-harness](https://github.com/epicsagas/epic-harness)(un plugin Claude Code auto-évolutif), ou tout harness personnalisé. Au lieu de polluer votre configuration par défaut, isolez chaque framework dans son propre mode :

```bash
# Créer un mode dédié pour le framework
claudy mode create gstack

# Copier ou lier la configuration du framework dans le répertoire du mode
cp -r /chemin/vers/gstack/.claude/. ~/.claudy/modes/gstack/

# Lancer Claude avec ce framework actif
claudy <profil> gstack
```

Chaque répertoire de mode est un `CLAUDE_CONFIG_DIR` autonome, donc les frameworks ne sont jamais en conflit entre eux ni avec votre configuration par défaut.

> **À utiliser avec [epic-harness](https://github.com/epicsagas/epic-harness).** Claudy gère la couche opérationnelle — changement de fournisseur, isolation de configuration, ponts canal/agent —, tandis qu'epic-harness (3 commandes, 26 compétences à déclenchement automatique, auto-évolutif à partir de vos schémas d'échec) apporte l'intelligence de l'agent. Même famille `epicsagas`; une séparation claire des responsabilités entre les modes.

<details>
<summary>Référence des commandes</summary>

## Référence des commandes

### Commandes principales

- `claudy ls` (alias : `list`) : lister les profils configurés/résolus.
- `claudy setup [fournisseur]` (alias : `config`) : configuration interactive d'un fournisseur.
- `claudy show <profil>` (alias : `info`) : afficher les détails résolus du fournisseur.
- `claudy ping [profil]` (alias : `test`) : tester la connectivité du fournisseur.
- `claudy doctor` (alias : `status`) : afficher la version, les chemins et le nombre de profils.
- `claudy sync` (alias : `install`) : installer/synchroniser le binaire claudy.
- `claudy update` : mettre à jour claudy.
- `claudy uninstall` : supprimer les fichiers installés.
- `claudy mode <action> [nom]` : gérer les modes de configuration Claude.
- `claudy channel <sous-commande>` : gérer le pont de canaux.
- `claudy mcp` : exécuter en tant que serveur MCP pour le pont d'agents.
- `claudy analytics <sous-commande>` : tableau de bord d'analytique d'utilisation.
- `claudy session sanitize` : réparer les sessions contenant des blocs thinking invalides écrits par des fournisseurs non-Anthropic.

### Commandes de mode

```bash
claudy mode create <nom>
claudy mode ls
claudy mode remove <nom>
```

Règle de nommage des modes : `[a-z0-9][a-z0-9_-]*` (`mode` est réservé).

### Commandes de canal (pont optionnel)

```bash
claudy channel serve [--profile <profil>] [--listen <hôte:port>]
claudy channel start [--profile <profil>] [--listen <hôte:port>]
claudy channel stop
claudy channel restart [--profile <profil>] [--listen <hôte:port>]
claudy channel status
claudy channel add <telegram|slack|discord>
claudy channel remove <telegram|slack|discord>
claudy channel enable
claudy channel disable
```

`channel add` vous guide à travers le jeton du bot, les utilisateurs autorisés, le profil et le mappage des modes.

#### Plateformes prises en charge

| Plateforme | Ingestion | Boutons interactifs | Remarques |
|----------|-----------|-------------------|-------|
| Telegram | Long-polling + webhook | Clavier intégré | La plus complète |
| Slack | Webhook d'abonnement aux événements | Actions Block Kit | Vérification HMAC-SHA256 |
| Discord | Webhook d'interaction | Composants de ligne d'action | Vérification Ed25519 |

#### Commandes du bot de canal

Une fois en cours d'exécution, le bot répond à ces commandes dans le chat :

- `/help` — Afficher les commandes disponibles
- `/cancel` — Annuler la tâche en cours
- `/model` — Changer le modèle Claude (boutons interactifs)
- `/yolo` — Activer/désactiver les permissions automatiques
- `/status` — Afficher l'état de la session, le profil, le mode, la branche git et l'utilisation des jetons
- `/sessions` — Lister les sessions Claude récentes (avec boutons de changement)
- `/projects` — Lister les projets (avec boutons de navigation)
- `/new` — Démarrer une nouvelle session
- `/history` — Afficher l'historique des sessions récentes

Envoyez tout autre texte pour parler directement à Claude.

#### Invites de permission

Lorsque Claude demande l'autorisation d'utiliser un outil (exécuter une commande, modifier un fichier, etc.),
le bot envoie une invite interactive Autoriser/Refuser dans votre chat. Appuyer sur un bouton
renvoie la réponse à Claude et le traitement se poursuit automatiquement.

#### Secrets

Stockez les identifiants de canal dans `~/.claudy/secrets.env` (voir [Identifiants des fournisseurs](#provider-credentials-secretsenv) pour le format complet) :

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## Pont MCP pour les agents

Exécutez `claudy mcp` pour démarrer un serveur MCP basé sur stdio qui permet à Claude Code de déléguer des tâches à d'autres agents de codage IA installés localement.

```bash
claudy mcp run        # Démarrer le serveur MCP (appelé par Claude Code)
claudy mcp install    # Enregistrer claudy comme serveur MCP dans les paramètres de Claude Code
claudy mcp uninstall  # Supprimer claudy des paramètres MCP de Claude Code
```

`claudy mcp install` s'enregistre automatiquement dans `~/.claude/settings.json`. Lorsque vous créez un mode avec `claudy mode create <nom>`, il s'enregistre également dans le fichier de paramètres du mode. Aucune configuration manuelle nécessaire.

Pour un enregistrement manuel (ou dans un fichier `.claude/settings.json` au niveau du projet) :

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

### Exemple d'utilisation

Une fois enregistré, Claude Code peut déléguer des tâches comme ceci :

```
> Ask agy to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code sélectionne l'agent approprié, transmet le prompt et renvoie le résultat. Vous pouvez également spécifier un répertoire de travail :

```json
{ "agent": "agy", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### Vérifier l'enregistrement MCP

```bash
# Vérifier si claudy est enregistré
cat ~/.claude/settings.json | grep -A3 claudy

# Tester le serveur MCP manuellement
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### Agents pris en charge (détection automatique depuis PATH)

| Agent | Binaire | Commande headless |
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

### Agents personnalisés

Ajoutez des agents dans `~/.claudy/config.yaml` sous la clé `agents` (voir [Configuration](#configyaml-schema) pour le schéma complet) :

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

Une clé identique à celle d'un agent intégré remplace ses valeurs par défaut. `{prompt}` dans `args` est remplacé par la tâche réelle.

## Analyse d'utilisation

> **Note** : La fonctionnalité d'analytique est encore en cours de développement. Les comptages de jetons, les estimations de coûts et autres métriques peuvent ne pas être entièrement précis. Attendez-vous à des améliorations dans les prochaines versions.

```bash
claudy analytics dashboard         # Ouvrir le tableau de bord d'analytique local (Tauri 2)
claudy analytics ingest            # Ingérer les données de session depuis ~/.claude/projects/
claudy analytics ingest --full     # Tout ré-ingérer (ignorer les points de contrôle)
claudy analytics ingest --project mon-projet  # Ingérer un projet spécifique
claudy analytics recommend         # Afficher les recommandations d'utilisation dans le CLI
claudy analytics export            # Exporter les données d'analytique (JSON, 30 jours par défaut)
claudy analytics export --format csv --days 7  # Exporter en CSV pour les 7 derniers jours
claudy analytics sync-pricing      # Synchroniser les prix des modèles depuis models.dev et la page de tarification Anthropic
claudy analytics recalculate       # Recalculer tous les coûts avec les dernières données de tarification
claudy analytics insights          # Générer un résumé JSON compact des insights (par défaut : 7 jours)
claudy analytics insights --days 14  # Analyser les 14 derniers jours
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # Plage de dates spécifique
claudy analytics insights --project mon-projet  # Filtrer par projet
```

### Dans Claude Code : `/analytics-insights`

Le moyen le plus rapide d'analyser votre utilisation est directement dans Claude Code. La compétence `analytics-insights` est automatiquement disponible — demandez simplement naturellement :

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude exécute `claudy analytics insights`, analyse le JSON et renvoie un rapport structuré avec :

- **Tendances des coûts** — dépenses quotidiennes/hebdomadaires avec détection des pics
- **Répartition des modèles** — quels modèles vous utilisez et ce qu'ils coûtent par session
- **Modèles d'outils** — outils les plus utilisés, taux d'erreur, observations d'efficacité
- **Performance du cache** — taux de réussite et économies estimées
- **Recommandations actionnables** — suggestions spécifiques comme « acheminer les tâches simples vers turbo » avec des économies estimées en dollars

Exemple de sortie (voir [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json) pour les données brutes) :

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

Pas de commandes manuelles, pas de changement de contexte. Posez des questions à Claude sur votre utilisation et obtenez des réponses instantanément.

### Ce que l'analytique suit

- **Jetons** : Tendances détaillées des jetons d'entrée, de sortie et de cache sur les 30 derniers jours, regroupés par modèle et par date.
- **Outils** : Analyse de distribution montrant quels outils Claude utilise le plus fréquemment, y compris les comptages d'appels, les taux d'erreur et le temps d'exécution moyen.
- **Coûts** : Estimation en temps réel des coûts d'utilisation basée sur la tarification réelle des jetons, y compris les prévisions quotidiennes/hebdomadaires/mensuelles et la détection de tendances (en hausse/stable/en baisse).
- **Conseils (Recommandations)** : Conseils d'optimisation basés sur les données, comme la détection de sessions à coûts élevés, la suggestion d'utiliser Haiku pour les tâches simples et l'identification des longues conversations qui pourraient bénéficier d'une synthèse de contexte.
- **Projets** : Mappe automatiquement les UUID de session cryptiques en noms de dossiers de projets lisibles par l'humain pour un meilleur contexte.

Les données sont stockées dans une base de données SQLite locale sous `~/.claudy/analytics/`. Le tableau de bord s'exécute comme une application locale haute performance Tauri 2 + Svelte. Utilisez le bouton **[Sync]** dans le tableau de bord pour rafraîchir instantanément les données depuis votre historique Claude CLI.

### Tableau de bord d'analytique
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="Tableau de bord d'analytique" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## Continuité de session inter-fournisseurs

Lors d'une session créée avec un fournisseur non-Anthropic (par ex. Z.AI / GLM), le fichier JSONL de session contient des blocs thinking avec une signature vide. La reprise de cette session via l'API Anthropic échoue avec :

```
API Error: 400 Invalid `signature` in `thinking` block
```

Claudy gère cela de deux façons :

**Automatique (pont de canal) :** Lorsque le serveur de canal reprend une session, il convertit silencieusement les blocs thinking à signature vide en blocs texte. Aucune action requise.

**Manuel (CLI) :** Utilisez `claudy session sanitize` avant de reprendre avec `claude --resume` :

```bash
# Interactif — sélectionner dans la liste des sessions problématiques
claudy session sanitize

# Filtrer par nom de projet
claudy session sanitize --project book-forge

# Traiter toutes les sessions en une fois
claudy session sanitize --all --yes
```

**Ce que fait la conversion :** Les blocs thinking à signature vide sont réécrits en blocs texte ordinaires, préservant le contenu du raisonnement. Les blocs avec une signature Anthropic valide ne sont pas modifiés.

**Limitation :** La continuité de session dépend de la compatibilité de l'historique de conversation. Un changement de fournisseur en cours de session peut entraîner de légères variations de contexte même après la correction.

---

## Fichiers et structure des répertoires

Par défaut, Claudy stocke les données dans :

```text
~/.claudy/
```

Fichiers et répertoires importants :

- `config.yaml` : configuration des fournisseurs, canaux et agents.
- `secrets.env` : identifiants des fournisseurs et bots.
- `launchers.json` : manifeste des lanceurs/liens symboliques.
- `modes/` : modes de configuration Claude.
- `session-patches/` : stockage des correctifs de session.
- `channel/` : état d'exécution du canal (`pid`, sessions, journal d'audit).
- `analytics/` : base de données SQLite d'analytique et points de contrôle.
- `cache/update.json` : cache des métadonnées de mise à jour.

## Variables d'environnement

- `CLAUDY_HOME` : remplacer le répertoire personnel de Claudy (par défaut : `~/.claudy`).
- `CLAUDE_CONFIG_DIR` : défini automatiquement par Claudy lors du lancement avec un mode.

## Flux de travail courants

### Configurer et lancer un fournisseur

```bash
claudy setup
claudy <profil>
```

### Utiliser un mode avec un fournisseur

```bash
claudy mode create travail
claudy <profil> travail --yolo
```

> `--yolo` est le raccourci claudy pour `--dangerously-skip-permissions`.

### Exécuter un framework Claude dédié dans son propre mode

Des frameworks comme gstack, superpowers, ecc ou notre [epic-harness](https://github.com/epicsagas/epic-harness) fournissent leur propre `CLAUDE.md`, compétences et agents. Gardez-les isolés :

```bash
# Configuration unique : créer le mode et l'initialiser avec la configuration du framework
claudy mode create gstack
cp -r /chemin/vers/gstack/.claude/. ~/.claudy/modes/gstack/

# Utilisation quotidienne : lancer Claude avec le framework actif
claudy <profil> gstack
```

Basculez entre les frameworks sans toucher à votre configuration par défaut :

```bash
claudy <profil> gstack      # framework gstack actif
claudy <profil> superpowers # framework superpowers actif
claudy <profil>             # votre configuration par défaut, inchangée
```

### Déléguer des tâches à d'autres agents via MCP

```bash
# 1) S'assurer que MCP est enregistré (se fait automatiquement au premier `claudy mcp`)
claudy mcp

# 2) Dans Claude Code, demander de déléguer à un agent installé :
#    "Ask agy to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnostiquer l'état de l'installation/configuration

```bash
claudy doctor
claudy ping
```

## Dépannage

- **`profile not recognized`** : exécutez `claudy ls` et choisissez un ID de profil listé.
- **Profil `not configured`** : exécutez `claudy setup <fournisseur>` pour ajouter des identifiants.
- **État du canal non sain** : exécutez `claudy channel status`, puis redémarrez avec `claudy channel stop` et `claudy channel start`.
- **Bot de canal ne répond pas** : vérifiez `~/.claudy/channel/logs/server.log` pour les erreurs. Vérifiez le jeton du bot dans `~/.claudy/secrets.env` et que `allowed_users` inclut votre identifiant d'utilisateur de chat.
- **L'invite de permission n'apparaît pas** : assurez-vous que Claude CLI ne fonctionne pas avec `--dangerously-skip-permissions`. L'invite ne se déclenche que lorsque Claude a besoin d'une approbation explicite pour l'utilisation d'un outil.
- **Binaire introuvable après l'installation** : consultez la note sur le PATH dans la section [Vérification](#verify).
- **Agent non affiché dans MCP** : assurez-vous que le binaire de l'agent est dans le `PATH` (`which agy`). Seuls les agents installés apparaissent dans `tools/list`.
- **Délai d'attente de l'agent dépassé** : augmentez le délai dans le champ agents de `config.yaml` (par défaut : 120s).
- **MCP non enregistré** : exécutez `claudy mcp` une fois manuellement, ou vérifiez `~/.claude/settings.json` pour l'entrée `mcpServers.claudy`.
- **Sortie de l'agent tronquée** : la sortie standard de l'agent est limitée à 10 Mo. Pour les grandes sorties, redirigez l'agent pour écrire dans un fichier à la place.
- **Données d'analytique manquantes** : exécutez `claudy analytics ingest` pour remplir depuis `~/.claude/projects/`. Utilisez `--full` pour tout ré-ingérer.
- **`400 Invalid signature in thinking block` à la reprise d'une session** : la session a été créée avec un fournisseur non-Anthropic (par ex. Z.AI). Exécutez `claudy session sanitize` pour convertir les blocs thinking invalides, puis reprenez normalement.

## Développement

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Tester le backend d'analytique (utilise la base de données locale)
cargo run --example test_dashboard --features analytics-ui

# Lancer le tableau de bord d'analytique (nécessite la fonctionnalité analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## Contribuer

Les contributions sont les bienvenues ! Voici comment commencer :

1. Forkez le dépôt et créez une branche de fonctionnalité.
2. Effectuez vos modifications avec des tests si nécessaire.
3. Exécutez `cargo test && cargo clippy -- -D warnings` avant de soumettre.
4. Ouvrez une Pull Request sur https://github.com/epicsagas/claudy.

Les rapports de bugs et les demandes de fonctionnalités sont les bienvenus via [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Remerciements

Ce projet a été inspiré par [Clother](https://github.com/jolehuit/clother), un lanceur multi-fournisseurs basé sur Go pour Claude CLI. Claudy est une implémentation Rust indépendante, repensée de zéro avec des gardes de session basées sur RAII, la transmission de signaux, les liens symboliques de lanceurs et des intégrations écosystémiques profondes incluant un **pont de canaux complet** (Telegram/Slack/Discord), le **pont MCP d'agents** pour la délégation inter-agents et un **tableau de bord d'analytique haute performance** construit avec Tauri 2. Ces ajouts reflètent la transition de Claudy d'un simple lanceur vers une boîte à outils opérationnelle complète pour les utilisateurs de Claude CLI.

## Licence

[Apache-2.0](../../LICENSE)
