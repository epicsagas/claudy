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

<p align="center"><b>Lançador multi-provedor moderno para o Claude CLI.</b></p>

---

<p align="center">
O Claudy permite que você execute o Claude com múltiplos providers por meio de uma interface de comandos unificada, mantendo as credenciais de cada provider e as sobreposições de configuração do Claude organizadas em um único diretório principal.
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../../assets/features-2048.png" alt="Por que Claudy" width="100%" />

## Por que Claudy

- **Lançamento multi-provedor**: alterne entre o provider integrado, Z.AI, alias do OpenRouter, Ollama e endpoints personalizados compatíveis com Anthropic.
- **Config modes**: isole a configuração do Claude (`CLAUDE.md`, `settings.json`, skills/plugins/agents) por Mode.
- **Resolução de Profile de provider**: unifica providers integrados, providers personalizados e aliases do OpenRouter.
- **Comportamento seguro do processo**: encaminha SIGINT/SIGTERM para o processo filho do Claude.
- **UX operacional**: comandos de instalação/atualização/desinstalação, verificações de status e testes de conectividade.
- **Channel bridge opcional**: execute um bot bridge local para Telegram, Slack e Discord com solicitações de permissão interativas.
- **Agent MCP bridge**: delegue tarefas do Claude Code para outros agentes de IA locais (Gemini, Codex, Aider, etc.) via MCP.
- **Analytics de uso**: ingere dados de sessão de `~/.claude/projects/`, rastreia o uso de tokens e custos por sessão/projeto, e exibe um dashboard local com recomendações.

## Status dos Providers

> O Claudy foi inspirado pelo [Clother](https://github.com/jolehuit/clother), um lançador multi-provedor baseado em Go para o Claude CLI. **Apenas o provider Z.AI foi completamente testado**. Todos os outros providers alternativos são experimentais e não foram testados — use-os por sua própria conta e risco.

| Provider | Status | Observações |
|---|---|---|
| Built-in (Anthropic) | ✅ Testado | Padrão |
| Z.AI | ✅ Testado | Completamente validado |
| OpenRouter alias | ⚠️ Experimental | Não testado — use por sua própria conta e risco |
| Ollama | ⚠️ Experimental | Não testado — use por sua própria conta e risco |
| Custom endpoint | ⚠️ Experimental | Não testado — use por sua própria conta e risco |

## Requisitos

- macOS ou Linux
- Toolchain do Rust (`cargo`) para compilar/instalar a partir do código fonte
- Claude CLI instalado e disponível no `PATH`

## Instalação

### Instalar pelo crates.io

**Binário pré-compilado (rápido, sem compilação)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**Qualquer plataforma — compilar a partir do código fonte**

```
cargo install claudy
```

**MacOS homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### Instalar a partir do código fonte local

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### Verificar

```bash
claudy --help
claudy --version
```

## Início Rápido

```bash
# 1) Listar profiles disponíveis/resolvidos
claudy ls

# 2) Configurar credenciais de forma interativa
claudy setup

# 3) Ver detalhes de um profile
claudy show <profile>

# 4) Executar o Claude com um profile
claudy <profile> [claude-args...]
```

## Conceitos Fundamentais

### Profile

Um alvo de lançamento que resolve metadados do provider e a estratégia de autenticação (provider integrado, alias do OpenRouter ou provider personalizado).

### Mode

Um diretório de configuração do Claude com nome, localizado em `~/.claudy/modes/<name>/`.

Quando você executa:

```bash
claudy <profile> <mode> [args...]
```

O Claudy define:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

para que o Claude leia os arquivos de configuração específicos do Mode.

## Referência de Comandos

### Comandos principais

- `claudy ls` (alias: `list`): lista os profiles configurados/resolvidos.
- `claudy setup [provider]` (alias: `config`): configuração interativa do provider.
- `claudy show <profile>` (alias: `info`): exibe os detalhes resolvidos do provider.
- `claudy ping [profile]` (alias: `test`): testa a conectividade do provider.
- `claudy doctor` (alias: `status`): exibe a versão, caminhos e quantidade de profiles.
- `claudy sync` (alias: `install`): instala/sincroniza o binário do claudy.
- `claudy update`: atualiza o claudy.
- `claudy uninstall`: remove os arquivos instalados.
- `claudy mode <action> [name]`: gerencia os Config Modes do Claude.
- `claudy channel <subcommand>`: gerencia o Channel bridge.
- `claudy mcp`: executa como servidor MCP para o Agent bridge.
- `claudy analytics <subcommand>`: dashboard de analytics de uso.

### Comandos de Mode

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Regra de nome do Mode: `[a-z0-9][a-z0-9_-]*` (`mode` é reservado).

### Comandos de Channel (bridge opcional)

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

`channel add` guia você pela configuração do token do bot, usuários permitidos, profile e mapeamento de Mode.

#### Plataformas suportadas

| Plataforma | Ingestão | Botões interativos | Observações |
|----------|-----------|-------------------|-------|
| Telegram | Long-polling + webhook | Teclado inline | Mais completo |
| Slack | Webhook de assinatura de eventos | Ações de Block Kit | Verificado com HMAC-SHA256 |
| Discord | Webhook de interação | Componentes de Action row | Verificado com Ed25519 |

#### Comandos do bot de Channel

Quando em execução, o bot responde a estes comandos no chat:

- `/help` — Exibe os comandos disponíveis
- `/cancel` — Cancela a tarefa atual
- `/model` — Altera o modelo do Claude (botões interativos)
- `/yolo` — Ativa/desativa a auto-aprovação de permissões
- `/status` — Exibe o status da sessão, profile, Mode, branch do git e uso de tokens
- `/sessions` — Lista as sessões recentes do Claude (com botões para alternar)
- `/projects` — Lista os projetos (com botões para navegar)
- `/new` — Inicia uma nova sessão
- `/history` — Exibe o histórico de sessões recentes

Envie qualquer outro texto para falar diretamente com o Claude.

#### Solicitações de permissão

Quando o Claude solicita aprovação para usar uma ferramenta (executar um comando, editar um arquivo, etc.), o bot envia uma solicitação interativa de Permitir/Negar para o seu chat. Tocar em um botão envia a resposta de volta ao Claude e o processamento continua automaticamente.

#### Segredos

Armazene as credenciais em `~/.claudy/secrets.env`:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP bridge

Execute `claudy mcp` para iniciar um servidor MCP baseado em stdio que permite ao Claude Code delegar tarefas para outros agentes de IA instalados localmente.

```bash
claudy mcp
```

Na primeira execução, o claudy se registra automaticamente em `~/.claude/settings.json`. Ao criar um Mode com `claudy mode create <name>`, ele também se registra no arquivo de configuração do Mode. Nenhuma configuração manual é necessária.

Para registrar manualmente (ou em um `.claude/settings.json` a nível de projeto):

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

#### Exemplo de uso

Uma vez registrado, o Claude Code pode delegar tarefas da seguinte forma:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

O Claude Code seleciona o agente apropriado, passa o prompt e retorna o resultado. Você também pode especificar um diretório de trabalho:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### Verificar o registro MCP

```bash
# Verificar se o claudy está registrado
cat ~/.claude/settings.json | grep -A3 claudy

# Testar o servidor MCP manualmente
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### Agentes suportados (detectados automaticamente pelo PATH)

| Agente | Binário | Comando headless |
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

#### Agentes personalizados

Adicione agentes em `~/.claudy/config.yaml`:

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

A mesma chave de um agente integrado substitui seus valores padrão. `{prompt}` em `args` é substituído pela tarefa real.

### Comandos de Analytics

> **Nota**: A funcionalidade de analytics ainda está em desenvolvimento. As contagens de tokens, estimativas de custos e outras métricas podem não ser completamente precisas. Melhorias são esperadas nas próximas versões.

```bash
claudy analytics dashboard         # Abrir o dashboard de analytics local (Tauri 2)
claudy analytics ingest            # Ingerir dados de sessão de ~/.claude/projects/
claudy analytics ingest --full     # Reingerir todos os arquivos (ignorar checkpoints)
claudy analytics ingest --project my-project  # Ingerir projeto específico
claudy analytics recommend         # Exibir recomendações de uso no CLI
claudy analytics export            # Exportar dados de analytics (JSON, padrão 30 dias)
claudy analytics export --format csv --days 7  # Exportar como CSV para os últimos 7 dias
```

O Analytics rastreia:

- **Tokens**: Tendências detalhadas de tokens de entrada, saída e cache nos últimos 30 dias, agrupados por modelo e data.
- **Tools**: Análise de distribuição mostrando quais ferramentas o Claude usa com mais frequência, incluindo contagens de chamadas, taxas de erro e tempo médio de execução.
- **Custo**: Estimativa em tempo real dos custos de uso com base nos preços reais de tokens, incluindo previsões diárias/semanais/mensais e detecção de tendências (crescente/estável/decrescente).
- **Dicas (Recomendações)**: Conselhos de otimização baseados em dados, como detecção de sessões de alto custo, sugestão do Haiku para tarefas simples e identificação de conversas longas que poderiam se beneficiar da sumarização de contexto.
- **Projetos**: Mapeia automaticamente UUIDs crípticos de sessão para nomes de pastas de projetos legíveis por humanos para melhor contexto.

Os dados são armazenados em um banco de dados SQLite local em `~/.claudy/analytics/`. O dashboard é executado como um aplicativo local de alto desempenho com Tauri 2 + Svelte. Use o botão **[Sync]** no dashboard para atualizar instantaneamente os dados do seu histórico do Claude CLI.

<img src="../../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## Arquivos e Estrutura de Diretórios

Por padrão, o Claudy armazena os dados em:

```text
~/.claudy/
```

Arquivos/diretórios importantes:

- `config.yaml`: configuração de provider, Channel e agente.
- `secrets.env`: credenciais do provider/bot.
- `launchers.json`: manifesto de lançadores/symlinks.
- `modes/`: Config Modes do Claude.
- `session-patches/`: armazenamento de patches de sessão.
- `channel/`: estado de execução do Channel (`pid`, sessões, log de auditoria).
- `analytics/`: banco de dados SQLite e checkpoints de analytics.
- `cache/update.json`: cache de metadados de atualização.

## Variáveis de Ambiente

- `CLAUDY_HOME`: substitui o diretório principal do Claudy (padrão: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: definido automaticamente pelo Claudy ao lançar com um Mode.

## Fluxos de Trabalho Comuns

### Configurar e lançar um provider

```bash
claudy setup
claudy <profile>
```

### Usar um Mode com um provider

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` é o atalho do claudy para `--dangerously-skip-permissions`.

### Delegar tarefas para outros agentes via MCP

```bash
# 1) Garantir que o MCP esteja registrado (acontece automaticamente no primeiro `claudy mcp`)
claudy mcp

# 2) No Claude Code, peça para delegar a qualquer agente instalado:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnosticar o estado de instalação/configuração

```bash
claudy doctor
claudy ping
```

## Solução de Problemas

- **`profile not recognized`**: execute `claudy ls` e escolha um ID de profile da lista.
- **Profile `not configured`**: execute `claudy setup <provider>` para adicionar credenciais.
- **Status do Channel não saudável**: execute `claudy channel status`, depois reinicie com `claudy channel stop` e `claudy channel start`.
- **Bot do Channel não responde**: verifique `~/.claudy/channel/logs/server.log` para erros. Verifique o token do bot em `~/.claudy/secrets.env` e se `allowed_users` inclui o ID do seu usuário de chat.
- **Solicitação de permissão não aparece**: certifique-se de que o Claude CLI não está sendo executado com `--dangerously-skip-permissions`. A solicitação só é acionada quando o Claude precisa de aprovação explícita para o uso de ferramentas.
- **Binário não encontrado após a instalação**: certifique-se de que o diretório bin do Claudy está no `PATH`, depois reinicie o seu shell.
- **Agente não aparece no MCP**: certifique-se de que o binário do agente está no `PATH` (`which gemini`). Apenas os agentes instalados aparecem em `tools/list`.
- **Timeout do agente**: aumente o timeout no campo agents de `config.yaml` (padrão: 120s).
- **MCP não registrado**: execute `claudy mcp` uma vez manualmente, ou verifique `~/.claude/settings.json` para a entrada `mcpServers.claudy`.
- **Saída do agente truncada**: a saída stdout do agente é limitada a 10MB. Para saídas grandes, redirecione o agente para escrever em um arquivo.
- **Dados de analytics ausentes**: execute `claudy analytics ingest` para popular a partir de `~/.claude/projects/`. Use `--full` para reingerir tudo.

## Desenvolvimento

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Testar o backend de analytics (usa BD local)
cargo run --example test_dashboard --features analytics-ui

# Lançar o dashboard de analytics (requer a feature analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## Contribuindo

Contribuições são bem-vindas! Veja como começar:

1. Faça um fork do repositório e crie uma branch de funcionalidade.
2. Faça suas alterações com testes onde for apropriado.
3. Execute `cargo test && cargo clippy -- -D warnings` antes de enviar.
4. Abra um Pull Request em https://github.com/epicsagas/claudy.

Relatórios de bugs e solicitações de funcionalidades são bem-vindos via [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Agradecimentos

Este projeto foi inspirado pelo [Clother](https://github.com/jolehuit/clother), um lançador multi-provedor baseado em Go para o Claude CLI. O Claudy é uma implementação independente em Rust, redesenhada do zero com guardas de sessão baseados em RAII, encaminhamento de sinais, symlinks de lançadores e integrações profundas com o ecossistema, incluindo um **Channel Bridge completo** (Telegram/Slack/Discord), o **Agent MCP Bridge** para delegação entre agentes, e um **dashboard de Analytics de alto desempenho** construído com Tauri 2. Essas adições refletem a transição do Claudy de um simples lançador para um kit de ferramentas operacional completo para usuários do Claude CLI.

## Licença

[Apache-2.0](../../LICENSE)
