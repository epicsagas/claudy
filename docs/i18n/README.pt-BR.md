<h1 align="center">claudy</h1>

<p align="center"><b>Um comando. Qualquer provedor. Controle total sobre o Claude CLI.</b></p>

<p align="center">
Chega de lidar com variáveis de ambiente e arquivos de configuração.<br/>
Claudy permite alternar entre Anthropic, Z.AI, OpenRouter, Ollama e endpoints personalizados com um único comando — mantendo credenciais, modos de configuração e frameworks do Claude isolados por perfil.
</p>

<p align="center">
<b>Multiprovedor · Isolamento de configuração · Ponte de canais · Ponte local de agentes · Análise de uso</b>
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
  <img alt="Por que Claudy" src="../assets/features-2048.png" width="100%">
</picture>

## Por que Claudy

| | Recurso | Por que importa |
|--|---------|----------------|
| 🔄 | Lançamento multiprovedor | Alterne entre Anthropic, Z.AI, OpenRouter, Ollama e endpoints personalizados em um comando |
| 📦 | Modos de configuração | Isole `CLAUDE.md`, configurações, skills e agentes por modo — sem contaminação cruzada |
| 🔗 | Ponte MCP de agentes | Delegue tarefas do Claude Code para agy, Codex, Aider e mais de 20 outros agentes |
| 💬 | Ponte de canais | Execute bots do Telegram, Slack e Discord com prompts de permissão interativos |
| 📊 | Análise de uso | Acompanhe o uso de tokens, custos e padrões de ferramentas com um dashboard Tauri local |
| 🔐 | Controle seguro de processos | Encaminhamento de SIGINT/SIGTERM, gravação atômica de configuração, armazenamento de credenciais com permissão 0600 |
| 🔀 | Continuidade de sessão entre provedores | Reparar automaticamente sessões do Z.AI/GLM para retomá-las com a API da Anthropic sem interrupções |
| 🛠️ | UX operacional | Instalação, atualização, desinstalação, diagnóstico, ping — tudo em um único binário |

## Provedores suportados

> Claudy foi inspirado por [Clother](https://github.com/jolehuit/clother), um lançador multiprovedor baseado em Go para o Claude CLI. O Z.AI é o provedor mais extensivamente testado. Se você encontrar problemas com outros provedores, por favor [abra uma issue](https://github.com/epicsagas/claudy/issues).

| Provedor | Status | Observações |
|---|---|---|
| Integrado (Anthropic) | ✅ Testado | Padrão |
| Z.AI | ✅ Testado | |
| Alias OpenRouter | ⚠️ Experimental | Não totalmente testado — relate problemas no GitHub |
| Ollama | ⚠️ Experimental | Não totalmente testado — relate problemas no GitHub |
| Endpoint personalizado | ⚠️ Experimental | Não totalmente testado — relate problemas no GitHub |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="demo" src="../assets/demo.gif" width="100%">
</picture>

## Início rápido

**1. Instalar**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

Sem Homebrew? Use o script de instalação:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Via Rust toolchain:

```bash
cargo binstall claudy   # binário pré-compilado (rápido)
cargo install claudy    # compilar a partir do código-fonte
```

**2. Configurar**

```bash
claudy install                        # inicializar diretórios, configuração, secrets
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. Executar**

```bash
claudy                                # provedor padrão
claudy zai                            # provedor Z.AI
claudy openrouter sonnet              # alias OpenRouter
```

**4. Atualizar**

```bash
brew upgrade claudy          # Homebrew
claudy update                # atualizador integrado
# ou execute novamente o script de instalação / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>Credenciais de provedores</summary>

| Variável | Provedor |
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
| `OPENROUTER_API_KEY` | OpenRouter (todos os aliases) |

Provedores personalizados usam a variável `api_key_env` definida em sua entrada `custom_providers`.

</details>

<details>
<summary>Esquema config.yaml</summary>

Toda a configuração fica em `~/.claudy/config.yaml`. Adicione apenas as seções necessárias — valores padrão são usados para qualquer campo omitido.

> Referência completa: [docs/config.md](../config.md)

```yaml
# Substituições de provedor — substitui o modelo e níveis de modelo padrão por provedor
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"                # → ANTHROPIC_DEFAULT_HAIKU_MODEL
      sonnet: "glm-5.1"               # → ANTHROPIC_DEFAULT_SONNET_MODEL
      opus: "glm-5"                   # → ANTHROPIC_DEFAULT_OPUS_MODEL

# Aliases OpenRouter — invoque como: claudy ou <alias>
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

# Provedores personalizados compatíveis com Anthropic — invoque como: claudy <slug>
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

# Política de compactação
compaction:
  auto_compact: true                   # padrão: true
  threshold: 0.8                       # 0.0–1.0, padrão: 0.8

# Substituições de janela de contexto por modelo
model_settings:
  deepseek-chat:
    max_context_tokens: 64000

# Ponte de canais — alternativa não interativa a `claudy channel add`
channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0           # 0 = ilimitado
  stream_timeout_secs: 1800

# Substituições de agentes
agents:
  aider:
    binary: "aider"
    args: ["--message", "{prompt}"]
    timeout: 300
```

</details>

---

## Conceitos principais

### Perfil

Um alvo de lançamento que resolve metadados do provedor + estratégia de autenticação (provedor integrado, alias OpenRouter ou provedor personalizado).

### Modo

Um diretório de configuração do Claude nomeado em `~/.claudy/modes/<name>/`.

Quando você executa:

```bash
claudy <profile> <mode> [args...]
```

Claudy define:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

para que o Claude leia os arquivos de configuração específicos do modo.

Modos também são ideais para **frameworks e toolkits dedicados do Claude** que trazem seu próprio `CLAUDE.md`, skills, agentes ou configurações — como [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code), nosso próprio [epic-harness](https://github.com/epicsagas/epic-harness)(um plugin do Claude Code que evolui sozinho), ou qualquer harness personalizado. Em vez de poluir sua configuração padrão, isole cada framework em seu próprio modo:

```bash
# Criar um modo dedicado para o framework
claudy mode create gstack

# Copiar ou criar symlink da configuração do framework no diretório do modo
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Executar o Claude com o framework ativo
claudy <profile> gstack
```

Cada diretório de modo é um `CLAUDE_CONFIG_DIR` independente, então os frameworks nunca conflitam entre si ou com sua configuração padrão.

> **Combina com o [epic-harness](https://github.com/epicsagas/epic-harness).** O Claudy cuida da camada operacional — troca de provedor, isolamento de configuração, pontes de canal/agente —, enquanto o epic-harness (3 comandos, 26 habilidades de gatilho automático, que evolui a partir dos seus padrões de falha) adiciona inteligência de agente. Mesma família `epicsagas`; uma separação clara de responsabilidades entre os modos.

<details>
<summary>Referência de comandos</summary>

## Referência de comandos

### Comandos principais

- `claudy ls` (alias: `list`): lista perfis configurados/resolvidos.
- `claudy setup [provider]` (alias: `config`): configuração interativa do provedor.
- `claudy show <profile>` (alias: `info`): exibe detalhes do provedor resolvido.
- `claudy ping [profile]` (alias: `test`): testa a conectividade do provedor.
- `claudy doctor` (alias: `status`): exibe versão, caminhos e contagem de perfis.
- `claudy sync` (alias: `install`): instala/sincroniza o binário do claudy.
- `claudy update`: atualiza o claudy.
- `claudy uninstall`: remove os arquivos instalados.
- `claudy mode <action> [name]`: gerencia modos de configuração do Claude.
- `claudy channel <subcommand>`: gerencia a ponte de canais.
- `claudy mcp`: executa como servidor MCP para ponte de agentes.
- `claudy analytics <subcommand>`: dashboard de análise de uso.
- `claudy session sanitize`: repara sessões com blocos thinking inválidos escritos por provedores não-Anthropic.

### Comandos de modo

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

Regra de nomenclatura de modos: `[a-z0-9][a-z0-9_-]*` (`mode` é reservado).

### Comandos de canal (ponte opcional)

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

`channel add` orienta você na configuração do token do bot, usuários permitidos, perfil e mapeamento de modo.

#### Plataformas suportadas

| Plataforma | Ingestão | Botões interativos | Observações |
|----------|-----------|-------------------|-------|
| Telegram | Long-polling + webhook | Inline keyboard | Mais completa |
| Slack | Event subscription webhook | Block Kit actions | Verificação HMAC-SHA256 |
| Discord | Interaction webhook | Action row components | Verificação Ed25519 |

#### Comandos do bot de canal

Quando em execução, o bot responde a estes comandos no chat:

- `/help` — Exibe os comandos disponíveis
- `/cancel` — Cancela a tarefa atual
- `/model` — Altera o modelo do Claude (botões interativos)
- `/yolo` — Ativa/desativa permissões automáticas
- `/status` — Exibe status da sessão, perfil, modo, branch git e uso de tokens
- `/sessions` — Lista sessões recentes do Claude (com botões para alternar)
- `/projects` — Lista projetos (com botões para navegar)
- `/new` — Inicia uma nova sessão
- `/history` — Exibe o histórico recente de sessões

Envie qualquer outro texto para conversar diretamente com o Claude.

#### Prompts de permissão

Quando o Claude solicita aprovação para usar uma ferramenta (executar um comando, editar um arquivo, etc.),
o bot envia um prompt interativo de Permitir/Negar para seu chat. Ao tocar em um botão,
a resposta é enviada de volta ao Claude e o processamento continua automaticamente.

#### Secrets

Armazene as credenciais do canal em `~/.claudy/secrets.env` (veja [Credenciais de provedores](#credenciais-de-provedores) para o formato completo):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## Ponte MCP de agentes

Execute `claudy mcp` para iniciar um servidor MCP baseado em stdio que permite ao Claude Code delegar tarefas para outros agentes de codificação por IA instalados localmente.

```bash
claudy mcp run        # Iniciar o servidor MCP (chamado pelo Claude Code)
claudy mcp install    # Registrar claudy como servidor MCP nas configurações do Claude Code
claudy mcp uninstall  # Remover claudy das configurações MCP do Claude Code
```

`claudy mcp install` se registra automaticamente em `~/.claude/settings.json`. Quando você cria um modo com `claudy mode create <name>`, ele também é registrado no arquivo de configurações do modo. Nenhuma configuração manual necessária.

Para registrar manualmente (ou em um `.claude/settings.json` de nível de projeto):

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

O Claude Code verá uma ferramenta `ask_agent` que expõe todos os agentes instalados.

### Exemplo de uso

Uma vez registrado, o Claude Code pode delegar tarefas como esta:

```
> Ask agy to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

O Claude Code seleciona o agente apropriado, passa o prompt e retorna o resultado. Você também pode especificar um diretório de trabalho:

```json
{ "agent": "agy", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### Verificar registro MCP

```bash
# Verificar se claudy está registrado
cat ~/.claude/settings.json | grep -A3 claudy

# Testar o servidor MCP manualmente
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### Agentes suportados (detectados automaticamente do PATH)

| Agente | Binário | Comando headless |
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

### Agentes personalizados

Adicione agentes em `~/.claudy/config.yaml` sob a chave `agents` (veja [Configuração](#esquema-configyaml) para o esquema completo):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

Uma chave com o mesmo nome de um agente integrado substitui seus padrões. `{prompt}` em `args` é substituído pela tarefa real.

## Análise de uso

> **Nota**: O recurso de análise ainda está em desenvolvimento. Contagens de tokens, estimativas de custos e outras métricas podem não ser totalmente precisas. Espere refinamentos nas próximas versões.

```bash
claudy analytics dashboard         # Abrir dashboard local de análise (Tauri 2)
claudy analytics ingest            # Ingerir dados de sessão de ~/.claude/projects/
claudy analytics ingest --full     # Reingerir todos os arquivos (ignorar checkpoints)
claudy analytics ingest --project my-project  # Ingerir projeto específico
claudy analytics recommend         # Exibir recomendações de uso no CLI
claudy analytics export            # Exportar dados de análise (JSON, padrão 30 dias)
claudy analytics export --format csv --days 7  # Exportar como CSV dos últimos 7 dias
claudy analytics sync-pricing      # Sincronizar preços de modelos do models.dev e página de preços da Anthropic
claudy analytics recalculate       # Recalcular todos os custos usando os dados de preços mais recentes
claudy analytics insights          # Gerar resumo compacto de insights em JSON (padrão: 7 dias)
claudy analytics insights --days 14  # Analisar últimos 14 dias
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # Intervalo de datas específico
claudy analytics insights --project my-project  # Filtrar por projeto
```

### Dentro do Claude Code: `/analytics-insights`

A forma mais rápida de analisar seu uso é diretamente dentro do Claude Code. O skill `analytics-insights` está disponível automaticamente — basta pedir naturalmente:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

O Claude executa `claudy analytics insights`, analisa o JSON e retorna um relatório estruturado com:

- **Tendências de custos** — gastos diários/semanais com detecção de picos
- **Distribuição de modelos** — quais modelos você usa e quanto custam por sessão
- **Padrões de ferramentas** — ferramentas mais usadas, taxas de erro, observações de eficiência
- **Desempenho do cache** — taxa de acerto e economia estimada
- **Recomendações acionáveis** — sugestões específicas como "roteie tarefas simples para turbo" com economia estimada em dólares

Exemplo de saída (veja [`docs/examples/analytics-insights-sample.json`](docs/examples/analytics-insights-sample.json) para dados brutos):

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

Sem comandos manuais, sem troca de contexto. Pergunte ao Claude sobre seu uso e receba respostas instantaneamente.

### O que a análise rastreia

- **Tokens**: Tendências detalhadas de tokens de entrada, saída e cache nos últimos 30 dias, agrupados por modelo e data.
- **Ferramentas**: Análise de distribuição mostrando quais ferramentas o Claude usa com mais frequência, incluindo contagem de chamadas, taxas de erro e tempo médio de execução.
- **Custo**: Estimativa em tempo real dos custos de uso com base no preço real dos tokens, incluindo previsões diárias/semanais/mensais e detecção de tendências (crescente/estável/decrescente).
- **Dicas (Recomendações)**: Conselhos de otimização baseados em dados, como detectar sessões de alto custo, sugerir Haiku para tarefas simples e identificar conversas longas que poderiam se beneficiar de resumo de contexto.
- **Projetos**: Mapeia automaticamente UUIDs de sessão crípticos para nomes legíveis de pastas de projetos para melhor contexto.

Os dados são armazenados em um banco de dados SQLite local em `~/.claudy/analytics/`. O dashboard é executado como um aplicativo local de alto desempenho em Tauri 2 + Svelte. Use o botão **[Sync]** no dashboard para atualizar instantaneamente os dados do histórico do seu Claude CLI.

### Dashboard de análise
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="Analytics Dashboard" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## Continuidade de sessão entre provedores

Ao trabalhar com um provedor não-Anthropic (ex.: Z.AI / GLM), o arquivo JSONL de sessão contém blocos thinking com assinatura vazia. Ao retomar essa sessão com a API da Anthropic, ocorre o seguinte erro:

```
API Error: 400 Invalid `signature` in `thinking` block
```

O Claudy trata isso de duas formas:

**Automática (bridge de canal):** Quando o servidor de canal retoma uma sessão, ele converte silenciosamente os blocos thinking com assinatura vazia em blocos de texto. Nenhuma ação necessária.

**Manual (CLI):** Use `claudy session sanitize` antes de retomar com `claude --resume`:

```bash
# Interativo — selecionar da lista de sessões com problema
claudy session sanitize

# Filtrar por nome do projeto
claudy session sanitize --project book-forge

# Processar todas as sessões de uma vez
claudy session sanitize --all --yes
```

**O que a conversão faz:** Blocos thinking com assinatura vazia são reescritos como blocos de texto simples, preservando o conteúdo do raciocínio. Blocos com assinatura Anthropic válida não são modificados.

**Limitação:** A continuidade da sessão depende da compatibilidade do histórico de conversa. Trocar de provedor no meio de uma sessão pode causar pequenas variações de contexto mesmo após a correção.

---

## Arquivos e estrutura de diretórios

Por padrão, Claudy armazena dados em:

```text
~/.claudy/
```

Arquivos/diretórios importantes:

- `config.yaml`: configuração de provedor + canal + agentes.
- `secrets.env`: credenciais de provedores/bots.
- `launchers.json`: manifesto de launchers/symlinks.
- `modes/`: modos de configuração do Claude.
- `session-patches/`: armazenamento de patches de sessão.
- `channel/`: estado de execução do canal (`pid`, sessões, log de auditoria).
- `analytics/`: banco de dados SQLite de análise e checkpoints.
- `cache/update.json`: cache de metadados de atualização.

## Variáveis de ambiente

- `CLAUDY_HOME`: substitui o diretório home do Claudy (padrão: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: definida automaticamente pelo Claudy ao executar com um modo.

## Fluxos de trabalho comuns

### Configurar e executar um provedor

```bash
claudy setup
claudy <profile>
```

### Usar um modo com um provedor

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` é o atalho do claudy para `--dangerously-skip-permissions`.

### Executar um framework dedicado do Claude em seu próprio modo

Frameworks como gstack, superpowers, ecc ou nosso [epic-harness](https://github.com/epicsagas/epic-harness) trazem seu próprio `CLAUDE.md`, skills e agentes. Mantenha-os isolados:

```bash
# Configuração única: criar o modo e inicializá-lo com a configuração do framework
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Uso diário: executar o Claude com o framework ativo
claudy <profile> gstack
```

Alterne entre frameworks sem modificar sua configuração padrão:

```bash
claudy <profile> gstack      # framework gstack ativo
claudy <profile> superpowers # framework superpowers ativo
claudy <profile>             # sua configuração padrão, inalterada
```

### Delegar tarefas para outros agentes via MCP

```bash
# 1) Certifique-se de que o MCP está registrado (ocorre automaticamente no primeiro `claudy mcp`)
claudy mcp

# 2) No Claude Code, peça para delegar para qualquer agente instalado:
#    "Ask agy to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnosticar estado de instalação/configuração

```bash
claudy doctor
claudy ping
```

## Solução de problemas

- **`profile not recognized`**: execute `claudy ls` e escolha um ID de perfil listado.
- **Perfil `not configured`**: execute `claudy setup <provider>` para adicionar credenciais.
- **Status do canal não saudável**: execute `claudy channel status`, depois reinicie com `claudy channel stop` e `claudy channel start`.
- **Bot do canal não responde**: verifique `~/.claudy/channel/logs/server.log` em busca de erros. Verifique o token do bot em `~/.claudy/secrets.env` e se `allowed_users` inclui seu ID de usuário no chat.
- **Prompt de permissão não aparece**: certifique-se de que o Claude CLI não está sendo executado com `--dangerously-skip-permissions`. O prompt só é acionado quando o Claude precisa de aprovação explícita para uso de ferramentas.
- **Binário não encontrado após instalação**: veja a nota sobre PATH na seção [Verificar](#verificar).
- **Agente não aparece no MCP**: certifique-se de que o binário do agente está no `PATH` (`which agy`). Apenas agentes instalados aparecem em `tools/list`.
- **Timeout do agente**: aumente o timeout no campo agents do `config.yaml` (padrão: 120s).
- **MCP não registrado**: execute `claudy mcp` uma vez manualmente, ou verifique `~/.claude/settings.json` pela entrada `mcpServers.claudy`.
- **Saída do agente truncada**: o stdout do agente é limitado a 10MB. Para saídas grandes, redirecione o agente para gravar em um arquivo.
- **Dados de análise ausentes**: execute `claudy analytics ingest` para preencher a partir de `~/.claude/projects/`. Use `--full` para reingerir tudo.
- **`400 Invalid signature in thinking block` ao retomar uma sessão**: a sessão foi criada com um provedor não-Anthropic (ex.: Z.AI). Execute `claudy session sanitize` para converter os blocos thinking inválidos e depois retome normalmente.

## Desenvolvimento

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Testar backend de análise (usa banco de dados local)
cargo run --example test_dashboard --features analytics-ui

# Abrir dashboard de análise (requer feature analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## Contribuindo

Contribuições são bem-vindas! Veja como começar:

1. Faça um fork do repositório e crie uma branch de feature.
2. Faça suas alterações com testes quando apropriado.
3. Execute `cargo test && cargo clippy -- -D warnings` antes de enviar.
4. Abra um Pull Request em https://github.com/epicsagas/claudy.

Relatórios de bugs e solicitações de funcionalidades são bem-vindos via [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Agradecimentos

Este projeto foi inspirado por [Clother](https://github.com/jolehuit/clother), um lançador multiprovedor baseado em Go para o Claude CLI. Claudy é uma implementação independente em Rust, redesenhada do zero com guards de sessão baseados em RAII, encaminhamento de sinais, symlinks de launchers e integrações profundas no ecossistema, incluindo uma **Ponte de Canais completa** (Telegram/Slack/Discord), a **Ponte MCP de Agentes** para delegação entre agentes e um **Dashboard de Análise de alto desempenho** construído com Tauri 2. Essas adições refletem a transição do Claudy de um simples lançador para um toolkit operacional abrangente para usuários do Claude CLI.

## Licença

[Apache-2.0](../../LICENSE)
