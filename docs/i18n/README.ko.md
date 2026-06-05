<h1 align="center">claudy</h1>

<p align="center"><b>하나의 명령. 모든 프로바이더. Claude CLI의 완전한 제어.</b></p>

<p align="center">
환경 변수와 설정 파일을 더 이상 헷갈리게 관리하지 마세요.<br/>
Claudy를 사용하면 Anthropic, Z.AI, OpenRouter, Ollama, 커스텀 엔드포인트를 단 한 번의 명령으로 전환할 수 있습니다 — 자격 증명, 설정 모드, Claude 프레임워크를 프로필별로 깔끔하게 분리합니다.
</p>

<p align="center">
<b>멀티 프로바이더 · 설정 격리 · 채널 브릿지 · 로컬 에이전트 브릿지 · 사용량 분석</b>
</p>

---

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> •
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
  <img alt="Claudy 기능 소개" src="../assets/features-2048.png" width="100%">
</picture>

## Claudy를 써야 하는 이유

| | 기능 | 왜 중요한가 |
|--|------|------------|
| 🔄 | 멀티 프로바이더 실행 | Anthropic, Z.AI, OpenRouter, Ollama, 커스텀 엔드포인트를 하나의 명령으로 전환 |
| 📦 | 설정 모드 | `CLAUDE.md`, 설정, 스킬, 에이전트를 모드별로 격리 — 교차 오염 없음 |
| 🔗 | 에이전트 MCP 브릿지 | Claude Code에서 Gemini, Codex, Aider 등 20개 이상의 에이전트로 작업 위임 |
| 💬 | 채널 브릿지 | Telegram, Slack, Discord 봇을 인터랙티브 권한 프롬프트와 함께 실행 |
| 📊 | 사용량 분석 | 토큰 사용량, 비용, 도구 패턴을 로컬 Tauri 대시보드에서 추적 |
| 🔐 | 안전한 프로세스 제어 | SIGINT/SIGTERM 전달, 원자적 설정 쓰기, 0600 자격 증명 저장 |
| 🔀 | 크로스 프로바이더 세션 연속성 | Z.AI/GLM으로 만든 세션을 Anthropic API로 이어서 작업할 수 있도록 자동 복구 |
| 🛠️ | 운영 UX | 설치, 업데이트, 제거, 진단, 핑 — 모든 것을 하나의 바이너리에서 |

## 지원 프로바이더

> Claudy는 Claude CLI용 Go 기반 멀티 프로바이더 런처인 [Clother](https://github.com/jolehuit/clother)에서 영감을 받았습니다. Z.AI가 가장 철저하게 테스트된 프로바이더입니다. 다른 프로바이더에 문제가 있으면 [이슈를 열어주세요](https://github.com/epicsagas/claudy/issues).

| 프로바이더 | 상태 | 비고 |
|---|---|---|
| 빌트인 (Anthropic) | ✅ 테스트 완료 | 기본값 |
| Z.AI | ✅ 테스트 완료 | |
| OpenRouter 별칭 | ⚠️ 실험적 | 완전히 테스트되지 않음 — GitHub에 이슈 보고 |
| Ollama | ⚠️ 실험적 | 완전히 테스트되지 않음 — GitHub에 이슈 보고 |
| 커스텀 엔드포인트 | ⚠️ 실험적 | 완전히 테스트되지 않음 — GitHub에 이슈 보고 |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="데모" src="../assets/demo.gif" width="100%">
</picture>

## 빠른 시작

**1. 설치**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

Homebrew가 없나요? 인스톨러 스크립트를 사용하세요:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Rust 툴체인으로 설치:

```bash
cargo binstall claudy   # 사전 빌드된 바이너리 (빠름)
cargo install claudy    # 소스에서 빌드
```

**2. 설정**

```bash
claudy install                        # 디렉토리, 설정, 시크릿 초기화
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. 실행**

```bash
claudy                                # 기본 프로바이더
claudy zai                            # Z.AI 프로바이더
claudy openrouter sonnet              # OpenRouter 별칭
```

**4. 업데이트**

```bash
brew upgrade claudy          # Homebrew
claudy update                # 빌트인 업데이터
# 또는 인스톨러 스크립트 재실행 / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>프로바이더 자격 증명</summary>

| 변수 | 프로바이더 |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (네이티브) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI 중국 |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax 중국 |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `ALIBABA_API_KEY` | Alibaba Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (모든 별칭) |

커스텀 프로바이더는 `custom_providers` 항목에 정의된 `api_key_env` 변수를 사용합니다.

</details>

<details>
<summary>config.yaml 스키마</summary>

모든 설정은 `~/.claudy/config.yaml`에 있습니다. 필요한 섹션만 추가하세요 — 생략된 항목은 기본값이 사용됩니다.

> 전체 레퍼런스: [docs/i18n/config.ko.md](config.ko.md)

```yaml
# 프로바이더 오버라이드 — 프로바이더별 기본 모델 및 모델 티어 오버라이드
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"                # → ANTHROPIC_DEFAULT_HAIKU_MODEL
      sonnet: "glm-5.1"               # → ANTHROPIC_DEFAULT_SONNET_MODEL
      opus: "glm-5"                   # → ANTHROPIC_DEFAULT_OPUS_MODEL

# OpenRouter 별칭 — claudy <별칭>으로 실행
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

# 커스텀 Anthropic 호환 프로바이더 — claudy <slug>로 실행
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

# 압축 정책
compaction:
  auto_compact: true                   # 기본값: true
  threshold: 0.8                       # 0.0–1.0, 기본값: 0.8

# 모델별 컨텍스트 윈도우 오버라이드
model_settings:
  deepseek-chat:
    max_context_tokens: 64000

# 채널 브릿지 — `claudy channel add`의 비대안형 대안
channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0           # 0 = 무제한
  stream_timeout_secs: 1800

# 에이전트 오버라이드
agents:
  aider:
    binary: "aider"
    args: ["--message", "{prompt}"]
    timeout: 300
```

</details>

---

## 핵심 개념

### 프로필

빌트인 프로바이더, OpenRouter 별칭, 또는 커스텀 프로바이더의 메타데이터 + 인증 전략을 해석하는 실행 대상입니다.

### 모드

`~/.claudy/modes/<name>/`에 있는 이름이 지정된 Claude 설정 디렉토리입니다.

다음과 같이 실행하면:

```bash
claudy <profile> <mode> [args...]
```

Claudy는 다음을 설정합니다:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

이렇게 하면 Claude가 모드별 설정 파일을 읽습니다.

모드는 자체 `CLAUDE.md`, 스킬, 에이전트 또는 설정을 제공하는 **전용 Claude 프레임워크 및 툴킷**에도 자연스럽게 활용할 수 있습니다 — 예를 들어 [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code) 또는 커스텀 하네스. 기본 설정을 오염시키는 대신 각 프레임워크를 자체 모드로 격리하세요:

```bash
# 프레임워크용 전용 모드 생성
claudy mode create gstack

# 프레임워크 설정을 모드 디렉토리에 복사 또는 심볼릭
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 해당 프레임워크가 활성화된 상태로 Claude 실행
claudy <profile> gstack
```

각 모드 디렉토리는 자체적인 `CLAUDE_CONFIG_DIR`이므로 프레임워크 간에 또는 기본 설정과 충돌이 발생하지 않습니다.

<details>
<summary>명령어 참조</summary>

## 명령어 참조

### 주요 명령어

- `claudy ls` (별칭: `list`): 설정된/해석된 프로필 나열.
- `claudy setup [provider]` (별칭: `config`): 대화형 프로바이더 설정.
- `claudy show <profile>` (별칭: `info`): 해석된 프로바이더 세부 정보 표시.
- `claudy ping [profile]` (별칭: `test`): 프로바이더 연결 테스트.
- `claudy doctor` (별칭: `status`): 버전, 경로, 프로필 수 표시.
- `claudy sync` (별칭: `install`): claudy 바이너리 설치/동기화.
- `claudy update`: claudy 업데이트.
- `claudy uninstall`: 설치된 파일 제거.
- `claudy mode <action> [name]`: Claude 설정 모드 관리.
- `claudy channel <subcommand>`: 채널 브릿지 관리.
- `claudy mcp`: 에이전트 브릿지용 MCP 서버로 실행.
- `claudy analytics <subcommand>`: 사용량 분석 대시보드.
- `claudy session sanitize`: 비 Anthropic 프로바이더의 잘못된 thinking 블록이 있는 세션을 복구합니다.

### 모드 명령어

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

모드 이름 규칙: `[a-z0-9][a-z0-9_-]*` (`mode`는 예약어).

### 채널 명령어 (선택 브릿지)

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

`channel add`는 봇 토큰, 허용 사용자, 프로필, 모드 매핑을 안내합니다.

#### 지원 플랫폼

| 플랫폼 | 수집 방식 | 인터랙티브 버튼 | 비고 |
|----------|-----------|----------------|-------|
| Telegram | 롱폴링 + 웹훅 | 인라인 키보드 | 가장 완성도 높음 |
| Slack | 이벤트 구독 웹훅 | Block Kit 액션 | HMAC-SHA256 검증 |
| Discord | 인터랙션 웹훅 | 액션 로우 컴포넌트 | Ed25519 검증 |

#### 채널 봇 명령어

실행 중에는 봇이 채팅에서 다음 명령어에 응답합니다:

- `/help` — 사용 가능한 명령어 표시
- `/cancel` — 현재 작업 취소
- `/model` — Claude 모델 변경 (인터랙티브 버튼)
- `/yolo` — 자동 권한 허용 토글
- `/status` — 세션 상태, 프로필, 모드, git 브랜치, 토큰 사용량 표시
- `/sessions` — 최근 Claude 세션 나열 (전환 버튼 포함)
- `/projects` — 프로젝트 나열 (탐색 버튼 포함)
- `/new` — 새 세션 시작
- `/history` — 최근 세션 기록 표시

다른 텍스트를 보내면 Claude와 직접 대화할 수 있습니다.

#### 권한 프롬프트

Claude가 도구 사용 승인을 요청하면(명령 실행, 파일 편집 등),
봇이 채팅에 인터랙티브 허용/거부 프롬프트를 보냅니다. 버튼을 탭하면
응답이 Claude로 다시 전송되고 처리가 자동으로 계속됩니다.

#### 시크릿

채널 자격 증명을 `~/.claudy/secrets.env`에 저장합니다 (전체 형식은 [프로바이더 자격 증명](#프로바이더-자격-증명-secretsenv) 참조):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## 에이전트 MCP 브릿지

`claudy mcp`를 실행하면 Claude Code가 로컬에 설치된 다른 AI 코딩 에이전트에 작업을 위임할 수 있는 stdio 기반 MCP 서버가 시작됩니다.

```bash
claudy mcp run        # MCP 서버 시작 (Claude Code가 호출)
claudy mcp install    # Claude Code 설정에 MCP 서버로 등록
claudy mcp uninstall  # Claude Code MCP 설정에서 제거
```

`claudy mcp install`은 자동으로 `~/.claude/settings.json`에 등록합니다. `claudy mode create <name>`으로 모드를 만들면 해당 모드의 설정 파일에도 등록됩니다. 수동 설정이 필요 없습니다.

수동으로 등록하려면 (또는 프로젝트 수준 `.claude/settings.json`에):

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

Claude Code는 설치된 모든 에이전트를 노출하는 `ask_agent` 도구를 인식합니다.

### 사용 예시

등록되면 Claude Code는 다음과 같이 작업을 위임할 수 있습니다:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code가 적절한 에이전트를 선택하고 프롬프트를 전달하며 결과를 반환합니다. 작업 디렉토리를 지정할 수도 있습니다:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### MCP 등록 확인

```bash
# claudy가 등록되어 있는지 확인
cat ~/.claude/settings.json | grep -A3 claudy

# MCP 서버 수동 테스트
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### 지원 에이전트 (PATH에서 자동 감지)

| 에이전트 | 바이너리 | 헤드리스 명령어 |
|-------|--------|----------------|
| Antigravity | `gemini` | `gemini -p "..." --output-format text` |
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

### 커스텀 에이전트

`~/.claudy/config.yaml`의 `agents` 키에 에이전트를 추가합니다 (전체 스키마는 [설정](#configyaml-스키마) 참조):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

빌트인 에이전트와 같은 키를 사용하면 기본값을 오버라이드합니다. `args`의 `{prompt}`는 실제 작업으로 교체됩니다.

## 사용량 분석

> **참고**: 분석 기능은 아직 작업 중입니다. 토큰 수, 비용 추정치 및 기타 지표가 완전히 정확하지 않을 수 있습니다. 향후 릴리스에서 개선될 예정입니다.

```bash
claudy analytics dashboard         # 로컬 분석 대시보드 열기 (Tauri 2)
claudy analytics ingest            # ~/.claude/projects/에서 세션 데이터 수집
claudy analytics ingest --full     # 모든 파일 재수집 (체크포인트 무시)
claudy analytics ingest --project my-project  # 특정 프로젝트 수집
claudy analytics recommend         # CLI에서 사용량 권장 사항 표시
claudy analytics export            # 분석 데이터 내보내기 (JSON, 기본 30일)
claudy analytics export --format csv --days 7  # 최근 7일 CSV로 내보내기
claudy analytics sync-pricing      # models.dev 및 Anthropic 가격 페이지에서 모델 가격 동기화
claudy analytics recalculate       # 최신 가격 데이터로 모든 비용 재계산
claudy analytics insights          # 간결한 JSON 인사이트 요약 생성 (기본: 7일)
claudy analytics insights --days 14  # 최근 14일 분석
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # 특정 날짜 범위
claudy analytics insights --project my-project  # 프로젝트별 필터
```

### Claude Code 안에서: `/analytics-insights`

사용량을 분석하는 가장 빠른 방법은 Claude Code 안에서 직접 하는 것입니다. `analytics-insights` 스킬이 자동으로 사용 가능합니다 — 자연스럽게 물어보세요:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude가 `claudy analytics insights`를 실행하고 JSON을 분석하여 구조화된 보고서를 반환합니다:

- **비용 추세** — 일/주간 지출과 급증 감지
- **모델 분포** — 어떤 모델을 사용하는지, 세션당 비용
- **도구 패턴** — 가장 많이 사용하는 도구, 오류율, 효율성 관찰
- **캐시 성능** — 적중률 및 예상 절감액
- **실행 가능한 권장 사항** — "단순 작업은 turbo로 라우팅" 등 구체적인 제안과 예상 달러 절감액

출력 예시 (원시 데이터는 [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json) 참조):

```
#### 요약
81개 세션, 총 $481 지출, 일평균 $68.7. 비용이 급격히
증가하는 추세 — 최근 3 영업일 일평균 $97.

#### 권장 사항
1. 단순 작업은 glm-5-turbo로 라우팅 — 예상 절감액: ~$90/월
2. $1.91/턴 이상 세션 조사 (평균 비용/턴의 6배)
3. harness 오버헤드 감소 — TaskCreate/Update가 약 1,000회 호출됨
```

수동 명령어 없이, 컨텍스트 전환 없이. Claude에게 사용량에 대해 물어보면 즉시 답변을 받습니다.

### 분석이 추적하는 항목

- **토큰**: 최근 30일간 입력, 출력, 캐시 토큰의 상세 추세 (모델 및 날짜별 그룹화)
- **도구**: Claude가 가장 자주 사용하는 도구의 분포 분석 (호출 수, 오류율, 평균 실행 시간 포함)
- **비용**: 실제 토큰 가격 기반 실시간 사용량 비용 추정 (일/주/월 예측 및 추세 감지 포함)
- **팁 (권장 사항)**: 고비용 세션 감지, 단순 작업에 Haiku 제안, 문맥 요약이 필요한 긴 대화 식별 등 데이터 기반 최적화 조언
- **프로젝트**: 암호화된 세션 UUID를 읽기 쉬운 프로젝트 폴더 이름으로 자동 매핑

데이터는 `~/.claudy/analytics/`의 로컬 SQLite 데이터베이스에 저장됩니다. 대시보드는 고성능 로컬 Tauri 2 + Svelte 앱으로 실행됩니다. 대시보드에서 **[Sync]** 버튼을 사용하여 Claude CLI 기록에서 즉시 데이터를 새로고침하세요.

### 분석 대시보드
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="분석 대시보드" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## 크로스 프로바이더 세션 연속성

Z.AI / GLM 등 비 Anthropic 프로바이더로 작업한 세션에는 빈 signature를 가진 thinking 블록이 기록됩니다. 해당 세션을 Anthropic API로 재개하면 다음 오류가 발생합니다:

```
API Error: 400 Invalid `signature` in `thinking` block
```

Claudy는 두 가지 방법으로 이 문제를 처리합니다:

**자동 (채널 브릿지):** 채널 서버가 세션을 재개할 때, 빈 signature를 가진 thinking 블록을 자동으로 일반 텍스트 블록으로 변환합니다. 별도 조치 불필요.

**수동 (CLI):** `claude --resume`으로 직접 재개하기 전에 `claudy session sanitize`로 세션을 복구합니다:

```bash
# 인터랙티브 — 문제 있는 세션 목록에서 선택
claudy session sanitize

# 프로젝트 이름으로 필터링
claudy session sanitize --project book-forge

# 모든 문제 세션 일괄 처리
claudy session sanitize --all --yes
```

**변환 방식:** 빈 signature의 thinking 블록이 일반 텍스트 블록으로 재작성됩니다. 추론 내용은 텍스트로 보존되며 세션 파일은 원자적으로 업데이트됩니다. 유효한 Anthropic signature가 있는 블록은 변경되지 않습니다.

**제한 사항:** 세션 연속성은 대화 기록의 호환성에 따라 달라집니다. 세션 도중 프로바이더를 전환하면 sanitization 이후에도 미묘한 맥락 변화가 발생할 수 있습니다.

---

## 파일 및 디렉토리 구조

기본적으로 Claudy는 다음 위치에 데이터를 저장합니다:

```text
~/.claudy/
```

주요 파일/디렉토리:

- `config.yaml`: 프로바이더 + 채널 + 에이전트 설정.
- `secrets.env`: 프로바이더/봇 자격 증명.
- `launchers.json`: 런처/심볼릭 매니페스트.
- `modes/`: Claude 설정 모드.
- `session-patches/`: 세션 패치 저장소.
- `channel/`: 채널 런타임 상태 (`pid`, 세션, 감사 로그).
- `analytics/`: 분석 SQLite 데이터베이스 및 체크포인트.
- `cache/update.json`: 업데이트 메타데이터 캐시.

## 환경 변수

- `CLAUDY_HOME`: Claudy 홈 디렉토리 오버라이드 (기본값: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: 모드로 실행할 때 Claudy가 자동 설정.

## 일반 워크플로우

### 프로바이더 설정 및 실행

```bash
claudy setup
claudy <profile>
```

### 모드와 함께 프로바이더 사용

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo`는 claudy의 `--dangerously-skip-permissions` 줄임말입니다.

### 전용 Claude 프레임워크를 자체 모드에서 실행

gstack, superpowers, ecc 같은 프레임워크는 자체 `CLAUDE.md`, 스킬, 에이전트를 제공합니다. 격리해서 유지하세요:

```bash
# 일회성 설정: 모드를 만들고 프레임워크 설정으로 시드
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 일상 사용: 프레임워크가 활성화된 상태로 Claude 실행
claudy <profile> gstack
```

기본 설정을 건드리지 않고 프레임워크 간에 전환:

```bash
claudy <profile> gstack      # gstack 프레임워크 활성
claudy <profile> superpowers # superpowers 프레임워크 활성
claudy <profile>             # 기본 설정, 변경 없음
```

### MCP를 통해 다른 에이전트로 작업 위임

```bash
# 1) MCP가 등록되어 있는지 확인 (첫 `claudy mcp` 시 자동 등록)
claudy mcp

# 2) Claude Code에서 설치된 에이전트에 작업 위임:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### 설치/설정 상태 진단

```bash
claudy doctor
claudy ping
```

## 문제 해결

- **`profile not recognized`**: `claudy ls`를 실행하고 나열된 프로필 ID를 선택하세요.
- **`not configured` 프로필**: `claudy setup <provider>`를 실행하여 자격 증명을 추가하세요.
- **채널 상태 비정상**: `claudy channel status`를 실행한 후 `claudy channel stop` 및 `claudy channel start`로 재시작하세요.
- **채널 봇 응답 없음**: `~/.claudy/channel/logs/server.log`에서 오류를 확인하세요. `~/.claudy/secrets.env`의 봇 토큰과 `allowed_users`에 채팅 사용자 ID가 포함되어 있는지 확인하세요.
- **권한 프롬프트가 나타나지 않음**: Claude CLI가 `--dangerously-skip-permissions`로 실행되지 않았는지 확인하세요. 프롬프트는 Claude가 도구 사용에 대해 명시적 승인이 필요할 때만 트리거됩니다.
- **설치 후 바이너리를 찾을 수 없음**: [확인](#verify) 섹션의 PATH 참고를 참조하세요.
- **에이전트가 MCP에 표시되지 않음**: 에이전트 바이너리가 `PATH`에 있는지 확인하세요 (`which gemini`). 설치된 에이전트만 `tools/list`에 나타납니다.
- **에이전트 시간 초과**: `config.yaml`의 agents 필드에서 시간 초과를 늘리세요 (기본값: 120초).
- **MCP가 등록되지 않음**: `claudy mcp`를 수동으로 한 번 실행하거나 `~/.claude/settings.json`에서 `mcpServers.claudy` 항목을 확인하세요.
- **에이전트 출력 잘림**: 에이전트 stdout은 10MB로 제한됩니다. 큰 출력의 경우 에이전트가 파일에 쓰도록 리디렉션하세요.
- **분석 데이터 누락**: `claudy analytics ingest`를 실행하여 `~/.claude/projects/`에서 데이터를 채우세요. `--full`을 사용하여 모두 재수집하세요.
- **세션 재개 시 `400 Invalid signature in thinking block`**: Z.AI 등 비 Anthropic 프로바이더로 생성된 세션입니다. `claudy session sanitize`를 실행해 잘못된 thinking 블록을 변환한 후 정상적으로 재개하세요.

## 개발

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# 분석 백엔드 테스트 (로컬 DB 사용)
cargo run --example test_dashboard --features analytics-ui

# 분석 대시보드 실행 (analytics-ui 기능 필요)
cargo run --features analytics-ui -- analytics dashboard
```

## 기여

기여를 환영합니다! 시작하는 방법:

1. 저장소를 포크하고 기능 브랜치를 만드세요.
2. 적절한 테스트와 함께 변경 사항을 만드세요.
3. 제출 전에 `cargo test && cargo clippy -- -D warnings`를 실행하세요.
4. https://github.com/epicsagas/claudy 에서 풀 리퀘스트를 여세요.

버그 리포트와 기능 요청은 [GitHub Issues](https://github.com/epicsagas/claudy/issues)를 통해 환영합니다.

## 감사의 글

이 프로젝트는 Claude CLI용 Go 기반 멀티 프로바이더 런처인 [Clother](https://github.com/jolehuit/clother)에서 영감을 받았습니다. Claudy는 독립적인 Rust 구현으로, RAII 기반 세션 가드, 시그널 전달, 런처 심볼릭 링크를 포함하여 처음부터 재설계되었으며, **풀 기능 채널 브릿지**(Telegram/Slack/Discord), 크로스 에이전트 위임을 위한 **에이전트 MCP 브릿지**, Tauri 2로 구축된 **고성능 분석 대시보드**를 포함한 심층적인 생태계 통합을 제공합니다. 이러한 추가 기능은 Claudy가 단순한 런처에서 Claude CLI 사용자를 위한 포괄적인 운영 툴킷으로 전환했음을 반영합니다.

## 라이선스

[Apache-2.0](../../LICENSE)
