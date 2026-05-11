<h1 align="center">claudy</h1>

<p align="center"><b>Un comando. Cualquier proveedor. Control total sobre Claude CLI.</b></p>

<p align="center">
Deja de pelear con variables de entorno y archivos de configuración.<br/>
Claudy te permite cambiar entre Anthropic, Z.AI, OpenRouter, Ollama y endpoints personalizados con un solo comando — manteniendo credenciales, modos de configuración y frameworks de Claude claramente aislados por perfil.
</p>

<p align="center">
<b>Multi-proveedor · Aislamiento de configuración · Puente de canales · Puente local de agentes · Análisis de uso</b>
</p>

---

<p align="center">
  <a href="README.ko.md">🇰🇷 한국어</a> •
  <a href="README.zh-Hans.md">🇨🇳 中文</a> •
  <a href="README.ja.md">🇯🇵 日本語</a> •
  <a href="README.de.md">🇩🇪 Deutsch</a> •
  <a href="README.fr.md">🇫🇷 Français</a> •
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
  <img alt="Por qué Claudy" src="../assets/features-2048.png" width="100%">
</picture>

## Por qué Claudy

| | Función | Por qué importa |
|--|---------|----------------|
| 🔄 | Lanzamiento multi-proveedor | Cambia entre Anthropic, Z.AI, OpenRouter, Ollama y endpoints personalizados con un solo comando |
| 📦 | Modos de configuración | Aísla `CLAUDE.md`, ajustes, skills y agentes por modo — sin contaminación cruzada |
| 🔗 | Puente MCP de agentes | Delega tareas desde Claude Code a Gemini, Codex, Aider y más de 20 agentes adicionales |
| 💬 | Puente de canales | Ejecuta bots de Telegram, Slack y Discord con solicitudes interactivas de permisos |
| 📊 | Análisis de uso | Rastrea uso de tokens, costos y patrones de herramientas con un panel local Tauri |
| 🔐 | Control seguro de procesos | Reenvío de SIGINT/SIGTERM, escrituras atómicas de configuración, almacenamiento de credenciales con permisos 0600 |
| 🛠️ | UX operacional | Instalar, actualizar, desinstalar, diagnosticar, verificar — todo desde un solo binario |

## Proveedores compatibles

> Claudy se inspiró en [Clother](https://github.com/jolehuit/clother), un lanzador multi-proveedor para Claude CLI escrito en Go. Z.AI ha sido el proveedor más probado exhaustivamente. Si encuentras problemas con otros proveedores, por favor [abre un issue](https://github.com/epicsagas/claudy/issues).

| Proveedor | Estado | Notas |
|---|---|---|
| Integrado (Anthropic) | ✅ Probado | Predeterminado |
| Z.AI | ✅ Probado | |
| Alias de OpenRouter | ⚠️ Experimental | No probado completamente — reporta problemas en GitHub |
| Ollama | ⚠️ Experimental | No probado completamente — reporta problemas en GitHub |
| Endpoint personalizado | ⚠️ Experimental | No probado completamente — reporta problemas en GitHub |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="demo" src="../assets/demo.gif" width="100%">
</picture>

## Inicio rápido

**1. Instalar**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

No tienes Homebrew? Usa el script de instalación:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Vía toolchain de Rust:

```bash
cargo binstall claudy   # binario precompilado (rápido)
cargo install claudy    # compilar desde el código fuente
```

**2. Configurar**

```bash
claudy install                        # inicializar directorios, config, secrets
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. Lanzar**

```bash
claudy                                # proveedor predeterminado
claudy zai                            # proveedor Z.AI
claudy openrouter sonnet              # alias de OpenRouter
```

**4. Actualizar**

```bash
brew upgrade claudy          # Homebrew
claudy update                # actualizador integrado
# o vuelve a ejecutar el script de instalación / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>Credenciales de proveedores</summary>

| Variable | Proveedor |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (nativo) |
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
| `OPENROUTER_API_KEY` | OpenRouter (todos los alias) |

Los proveedores personalizados usan la variable `api_key_env` definida en su entrada `custom_providers`.

</details>

<details>
<summary>Esquema config.yaml</summary>

Toda la configuración se encuentra en `~/.claudy/config.yaml`. Solo agrega las secciones que necesites — se usan valores predeterminados para todo lo omitido.

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

## Conceptos clave

### Perfil

Un objetivo de lanzamiento que resuelve los metadatos del proveedor y la estrategia de autenticación (proveedor integrado, alias de OpenRouter o proveedor personalizado).

### Modo

Un directorio de configuración de Claude con nombre ubicado en `~/.claudy/modes/<nombre>/`.

Cuando ejecutas:

```bash
claudy <perfil> <modo> [args...]
```

Claudy establece:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<modo>/
```

para que Claude lea los archivos de configuración específicos del modo.

Los modos también son ideales para **frameworks y toolkits dedicados de Claude** que incluyen su propio `CLAUDE.md`, skills, agentes o ajustes — como [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code), o cualquier harness personalizado. En lugar de contaminar tu configuración predeterminada, aísla cada framework en su propio modo:

```bash
# Crea un modo dedicado para el framework
claudy mode create gstack

# Copia o enlaza la configuración del framework en el directorio del modo
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Lanza Claude con ese framework activo
claudy <perfil> gstack
```

Cada directorio de modo es un `CLAUDE_CONFIG_DIR` autónomo, por lo que los frameworks nunca entran en conflicto entre sí ni con tu configuración predeterminada.

<details>
<summary>Referencia de comandos</summary>

## Referencia de comandos

### Comandos principales

- `claudy ls` (alias: `list`): listar perfiles configurados/resueltos.
- `claudy setup [proveedor]` (alias: `config`): configuración interactiva del proveedor.
- `claudy show <perfil>` (alias: `info`): mostrar detalles resueltos del proveedor.
- `claudy ping [perfil]` (alias: `test`): probar la conectividad del proveedor.
- `claudy doctor` (alias: `status`): mostrar versión, rutas y cantidad de perfiles.
- `claudy sync` (alias: `install`): instalar/sincronizar el binario de claudy.
- `claudy update`: actualizar claudy.
- `claudy uninstall`: eliminar archivos instalados.
- `claudy mode <acción> [nombre]`: gestionar modos de configuración de Claude.
- `claudy channel <subcomando>`: gestionar el puente de canales.
- `claudy mcp`: ejecutar como servidor MCP para el puente de agentes.
- `claudy analytics <subcomando>`: panel de análisis de uso.

### Comandos de modo

```bash
claudy mode create <nombre>
claudy mode ls
claudy mode remove <nombre>
```

Regla de nombres de modo: `[a-z0-9][a-z0-9_-]*` (`mode` está reservado).

### Comandos de canal (puente opcional)

```bash
claudy channel serve [--profile <perfil>] [--listen <host:puerto>]
claudy channel start [--profile <perfil>] [--listen <host:puerto>]
claudy channel stop
claudy channel restart [--profile <perfil>] [--listen <host:puerto>]
claudy channel status
claudy channel add <telegram|slack|discord>
claudy channel remove <telegram|slack|discord>
claudy channel enable
claudy channel disable
```

`channel add` te guía a través del token del bot, usuarios permitidos, perfil y mapeo de modos.

#### Plataformas compatibles

| Plataforma | Ingesta | Botones interactivos | Notas |
|----------|-----------|-------------------|-------|
| Telegram | Long-polling + webhook | Teclado inline | La más completa |
| Slack | Webhook de suscripción de eventos | Acciones Block Kit | Verificado con HMAC-SHA256 |
| Discord | Webhook de interacciones | Componentes de filas de acciones | Verificado con Ed25519 |

#### Comandos del bot de canal

Una vez en ejecución, el bot responde a estos comandos en el chat:

- `/help` — Mostrar comandos disponibles
- `/cancel` — Cancelar tarea actual
- `/model` — Cambiar modelo de Claude (botones interactivos)
- `/yolo` — Activar/desactivar permisos automáticos
- `/status` — Mostrar estado de sesión, perfil, modo, rama git y uso de tokens
- `/sessions` — Listar sesiones recientes de Claude (con botones para cambiar)
- `/projects` — Listar proyectos (con botones para explorar)
- `/new` — Iniciar una nueva sesión
- `/history` — Mostrar historial reciente de sesiones

Envía cualquier otro texto para hablar directamente con Claude.

#### Solicitudes de permisos

Cuando Claude solicita aprobación para usar una herramienta (ejecutar un comando, editar un archivo, etc.),
el bot envía una solicitud interactiva de Permitir/Denegar a tu chat. Al pulsar un botón
se envía la respuesta de vuelta a Claude y el procesamiento continúa automáticamente.

#### Secrets

Almacena las credenciales del canal en `~/.claudy/secrets.env` (ver [Credenciales de proveedores](#credenciales-de-proveedores) para el formato completo):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## Puente MCP de agentes

Ejecuta `claudy mcp` para iniciar un servidor MCP basado en stdio que permite a Claude Code delegar tareas a otros agentes de codificación por IA instalados localmente.

```bash
claudy mcp run        # Iniciar el servidor MCP (llamado por Claude Code)
claudy mcp install    # Registrar claudy como servidor MCP en la configuración de Claude Code
claudy mcp uninstall  # Eliminar claudy de la configuración MCP de Claude Code
```

`claudy mcp install` se registra automáticamente en `~/.claude/settings.json`. Cuando creas un modo con `claudy mode create <nombre>`, también se registra en el archivo de ajustes del modo. No se necesita configuración manual.

Para registrar manualmente (o en un archivo `.claude/settings.json` a nivel de proyecto):

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

Claude Code verá una herramienta `ask_agent` que expone todos los agentes instalados.

### Ejemplo de uso

Una vez registrado, Claude Code puede delegar tareas como esta:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code selecciona el agente apropiado, pasa el prompt y devuelve el resultado. También puedes especificar un directorio de trabajo:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### Verificar registro MCP

```bash
# Verificar si claudy está registrado
cat ~/.claude/settings.json | grep -A3 claudy

# Probar el servidor MCP manualmente
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### Agentes compatibles (auto-detectados desde PATH)

| Agente | Binario | Comando headless |
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

### Agentes personalizados

Agrega agentes en `~/.claudy/config.yaml` bajo la clave `agents` (ver [Configuración](#esquema-configyaml) para el esquema completo):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

Si la clave coincide con un agente integrado, se sobreescriben sus valores predeterminados. `{prompt}` en `args` se reemplaza con la tarea real.

## Análisis de uso

> **Nota**: La función de análisis aún está en desarrollo. Los recuentos de tokens, estimaciones de costos y otras métricas pueden no ser completamente precisos. Espera mejoras en las próximas versiones.

```bash
claudy analytics dashboard         # Abrir panel local de análisis (Tauri 2)
claudy analytics ingest            # Ingerir datos de sesión desde ~/.claude/projects/
claudy analytics ingest --full     # Re-ingerir todos los archivos (ignorar checkpoints)
claudy analytics ingest --project mi-proyecto  # Ingerir proyecto específico
claudy analytics recommend         # Mostrar recomendaciones de uso en CLI
claudy analytics export            # Exportar datos de análisis (JSON, predeterminado 30 días)
claudy analytics export --format csv --days 7  # Exportar como CSV de los últimos 7 días
claudy analytics sync-pricing      # Sincronizar precios de modelos desde models.dev y la página de precios de Anthropic
claudy analytics recalculate       # Recalcular todos los costos usando los datos de precios más recientes
claudy analytics insights          # Generar resumen compacto de análisis en JSON (predeterminado: 7 días)
claudy analytics insights --days 14  # Analizar los últimos 14 días
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # Rango de fechas específico
claudy analytics insights --project mi-proyecto  # Filtrar por proyecto
```

### Dentro de Claude Code: `/analytics-insights`

La forma más rápida de analizar tu uso es directamente dentro de Claude Code. El skill `analytics-insights` está disponible automáticamente — solo preguntando de forma natural:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude ejecuta `claudy analytics insights`, analiza el JSON y devuelve un informe estructurado con:

- **Tendencias de costos** — gasto diario/semanal con detección de picos
- **Distribución de modelos** — qué modelos usas y cuánto cuestan por sesión
- **Patrones de herramientas** — herramientas más usadas, tasas de error, observaciones de eficiencia
- **Rendimiento de caché** — ratio de aciertos y ahorro estimado
- **Recomendaciones accionables** — sugerencias específicas como "enruta tareas simples a turbo" con ahorro estimado en dólares

Ejemplo de salida (ver [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json) para datos sin procesar):

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

Sin comandos manuales, sin cambiar de contexto. Pregunta a Claude sobre tu uso y obtén respuestas al instante.

### Qué rastrea el análisis

- **Tokens**: Tendencias detalladas de tokens de entrada, salida y caché durante los últimos 30 días, agrupados por modelo y fecha.
- **Herramientas**: Análisis de distribución que muestra qué herramientas usa Claude con más frecuencia, incluyendo recuentos de llamadas, tasas de error y tiempo promedio de ejecución.
- **Costos**: Estimación en tiempo real de los costos de uso basada en precios reales de tokens, incluyendo pronósticos diarios/semanales/mensuales y detección de tendencias (creciente/estable/decreciente).
- **Consejos (Recomendaciones)**: Consejos de optimización basados en datos, como detectar sesiones de alto costo, sugerir Haiku para tareas simples e identificar conversaciones largas que podrían beneficiarse de resumen de contexto.
- **Proyectos**: Mapea automáticamente UUIDs de sesión crípticos a nombres legibles de carpetas de proyectos para mejor contexto.

Los datos se almacenan en una base de datos SQLite local en `~/.claudy/analytics/`. El panel se ejecuta como una aplicación local de alto rendimiento con Tauri 2 + Svelte. Usa el botón **[Sync]** en el panel para actualizar instantáneamente los datos desde tu historial de Claude CLI.

### Panel de análisis
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="Panel de análisis" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## Archivos y estructura de directorios

Por defecto, Claudy almacena datos en:

```text
~/.claudy/
```

Archivos/directorios importantes:

- `config.yaml`: configuración de proveedor + canal + agentes.
- `secrets.env`: credenciales de proveedor/bot.
- `launchers.json`: manifiesto de launchers/symlinks.
- `modes/`: modos de configuración de Claude.
- `session-patches/`: almacenamiento de parches de sesión.
- `channel/`: estado de ejecución del canal (`pid`, sesiones, registro de auditoría).
- `analytics/`: base de datos SQLite de análisis y checkpoints.
- `cache/update.json`: caché de metadatos de actualización.

## Variables de entorno

- `CLAUDY_HOME`: sobreescribir el directorio principal de Claudy (predeterminado: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: establecido automáticamente por Claudy al lanzar con un modo.

## Flujos de trabajo comunes

### Configurar y lanzar un proveedor

```bash
claudy setup
claudy <perfil>
```

### Usar un modo con un proveedor

```bash
claudy mode create work
claudy <perfil> work --yolo
```

> `--yolo` es el atajo de claudy para `--dangerously-skip-permissions`.

### Ejecutar un framework de Claude dedicado en su propio modo

Frameworks como gstack, superpowers o ecc incluyen su propio `CLAUDE.md`, skills y agentes. Mantenlos aislados:

```bash
# Configuración inicial: crear el modo y cargar la configuración del framework
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Uso diario: lanzar Claude con el framework activo
claudy <perfil> gstack
```

Cambia entre frameworks sin tocar tu configuración predeterminada:

```bash
claudy <perfil> gstack      # framework gstack activo
claudy <perfil> superpowers # framework superpowers activo
claudy <perfil>             # tu configuración predeterminada, sin cambios
```

### Delegar tareas a otros agentes vía MCP

```bash
# 1) Asegurar que MCP está registrado (ocurre automáticamente en el primer `claudy mcp`)
claudy mcp

# 2) En Claude Code, pedir que delegue a cualquier agente instalado:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnosticar estado de instalación/configuración

```bash
claudy doctor
claudy ping
```

## Solución de problemas

- **`profile not recognized`**: ejecuta `claudy ls` y elige un ID de perfil de la lista.
- **Perfil `not configured`**: ejecuta `claudy setup <proveedor>` para agregar credenciales.
- **Estado del canal no saludable**: ejecuta `claudy channel status`, luego reinicia con `claudy channel stop` y `claudy channel start`.
- **Bot del canal no responde**: revisa `~/.claudy/channel/logs/server.log` en busca de errores. Verifica el token del bot en `~/.claudy/secrets.env` y que `allowed_users` incluya tu ID de usuario de chat.
- **Solicitud de permisos no aparece**: asegúrate de que Claude CLI no esté ejecutándose con `--dangerously-skip-permissions`. La solicitud solo se activa cuando Claude necesita aprobación explícita para el uso de herramientas.
- **Binario no encontrado después de instalar**: ver la nota sobre PATH en la sección [Verificar](#verify).
- **Agente no aparece en MCP**: asegúrate de que el binario del agente esté en `PATH` (`which gemini`). Solo los agentes instalados aparecen en `tools/list`.
- **Timeout del agente**: aumenta el timeout en el campo agents de `config.yaml` (predeterminado: 120s).
- **MCP no registrado**: ejecuta `claudy mcp` una vez manualmente, o revisa `~/.claude/settings.json` para la entrada `mcpServers.claudy`.
- **Salida del agente truncada**: la salida estándar del agente está limitada a 10MB. Para salidas grandes, redirige el agente para que escriba en un archivo.
- **Datos de análisis faltantes**: ejecuta `claudy analytics ingest` para poblar desde `~/.claude/projects/`. Usa `--full` para re-ingerir todo.

## Desarrollo

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Probar el backend de análisis (usa base de datos local)
cargo run --example test_dashboard --features analytics-ui

# Lanzar panel de análisis (requiere la característica analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## Contribuir

Las contribuciones son bienvenidas! Aquí te explicamos cómo empezar:

1. Haz un fork del repositorio y crea una rama de funcionalidad.
2. Realiza tus cambios con pruebas donde sea apropiado.
3. Ejecuta `cargo test && cargo clippy -- -D warnings` antes de enviar.
4. Abre un Pull Request en https://github.com/epicsagas/claudy.

Los reportes de bugs y solicitudes de funcionalidades son bienvenidos a través de [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Agradecimientos

Este proyecto se inspiró en [Clother](https://github.com/jolehuit/clother), un lanzador multi-proveedor para Claude CLI escrito en Go. Claudy es una implementación independiente en Rust, rediseñada desde cero con guards de sesión basados en RAII, reenvío de señales, symlinks de launcher e integraciones profundas con el ecosistema, incluyendo un **Puente de Canales con todas las funciones** (Telegram/Slack/Discord), el **Puente MCP de Agentes** para delegación entre agentes, y un **Panel de Análisis de alto rendimiento** construido con Tauri 2. Estas adiciones reflejan la transición de Claudy de un simple lanzador a un toolkit operacional integral para usuarios de Claude CLI.

## Licencia

[Apache-2.0](../../LICENSE)
