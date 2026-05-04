[← English](../../README.md)

<h1 align="center">claudy</h1>

<p align="center"><b>Claude CLI를 위한 현대적인 멀티 Provider 런처.</b></p>

---

<p align="center">
Claudy는 하나의 일관된 명령어 인터페이스로 여러 Provider에서 Claude를 실행할 수 있도록 도와주며, Provider 자격증명과 Claude 설정 오버레이를 단일 홈 디렉터리 아래에 정리하여 관리합니다.
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../../assets/features-2048.png" alt="Why Claudy" width="100%" />

## Claudy를 사용하는 이유

- **멀티 Provider 실행**: 빌트인, Z.AI, OpenRouter 별칭, Ollama, 커스텀 Anthropic 호환 엔드포인트 간 전환.
- **Config Mode**: Mode별로 Claude 설정(`CLAUDE.md`, `settings.json`, 스킬/플러그인/에이전트)을 격리.
- **Provider Profile 해석**: 빌트인 Provider, 커스텀 Provider, OpenRouter 별칭을 통합.
- **안전한 프로세스 동작**: 자식 Claude 프로세스에 SIGINT/SIGTERM을 전달.
- **운영 UX**: 설치/업데이트/제거 명령어, 상태 확인, 연결 테스트.
- **선택적 Channel 브릿지**: 대화형 권한 프롬프트와 함께 Telegram, Slack, Discord를 위한 로컬 봇 브릿지 실행.
- **에이전트 MCP 브릿지**: MCP를 통해 Claude Code에서 다른 로컬 AI 에이전트(Gemini, Codex, Aider 등)로 작업 위임.
- **사용량 분석**: `~/.claude/projects/`에서 세션 데이터를 수집하고, 세션/프로젝트별 토큰 사용량 및 비용을 추적하며, 권장 사항이 포함된 로컬 대시보드를 제공.

## Provider 현황

> Claudy는 Go 기반의 Claude CLI 멀티 Provider 런처인 [Clother](https://github.com/jolehuit/clother)에서 영감을 받았습니다. **Z.AI Provider만 완전히 테스트되었습니다**. 다른 대체 Provider는 모두 실험적이며 테스트되지 않았습니다 — 사용 시 위험을 감수하세요.

| Provider | 상태 | 비고 |
|---|---|---|
| 빌트인 (Anthropic) | ✅ 테스트 완료 | 기본값 |
| Z.AI | ✅ 테스트 완료 | 완전히 검증됨 |
| OpenRouter 별칭 | ⚠️ 실험적 | 테스트되지 않음 — 사용 시 위험 감수 필요 |
| Ollama | ⚠️ 실험적 | 테스트되지 않음 — 사용 시 위험 감수 필요 |
| 커스텀 엔드포인트 | ⚠️ 실험적 | 테스트되지 않음 — 사용 시 위험 감수 필요 |

## 요구 사항

- macOS 또는 Linux
- 소스에서 빌드/설치하려면 Rust 툴체인(`cargo`) 필요
- Claude CLI가 설치되어 있고 `PATH`에서 사용 가능해야 함

## 설치

### crates.io에서 설치

**사전 빌드된 바이너리 (빠름, 컴파일 불필요)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**모든 플랫폼 — 소스에서 빌드**

```
cargo install claudy
```

**macOS Homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### 로컬 소스에서 설치

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### 확인

```bash
claudy --help
claudy --version
```

## 빠른 시작

```bash
# 1) 사용 가능한/해석된 Profile 나열
claudy ls

# 2) 대화형으로 자격증명 설정
claudy setup

# 3) 하나의 Profile 세부 정보 확인
claudy show <profile>

# 4) Profile로 Claude 실행
claudy <profile> [claude-args...]
```

## 핵심 개념

### Profile

Provider 메타데이터 + 인증 전략(빌트인 Provider, OpenRouter 별칭 또는 커스텀 Provider)을 해석하는 실행 대상.

### Mode

`~/.claudy/modes/<name>/`에 위치한 명명된 Claude 설정 디렉터리.

다음을 실행하면:

```bash
claudy <profile> <mode> [args...]
```

Claudy가 설정합니다:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

그러면 Claude가 Mode별 설정 파일을 읽습니다.

## 명령어 참조

### 주요 명령어

- `claudy ls` (별칭: `list`): 설정된/해석된 Profile 나열.
- `claudy setup [provider]` (별칭: `config`): 대화형 Provider 설정.
- `claudy show <profile>` (별칭: `info`): 해석된 Provider 세부 정보 표시.
- `claudy ping [profile]` (별칭: `test`): Provider 연결 테스트.
- `claudy doctor` (별칭: `status`): 버전, 경로, Profile 수 표시.
- `claudy sync` (별칭: `install`): claudy 바이너리 설치/동기화.
- `claudy update`: claudy 업데이트.
- `claudy uninstall`: 설치된 파일 제거.
- `claudy mode <action> [name]`: Claude 설정 Mode 관리.
- `claudy channel <subcommand>`: Channel 브릿지 관리.
- `claudy mcp`: 에이전트 브릿지용 MCP 서버로 실행.
- `claudy analytics <subcommand>`: 사용량 분석 대시보드.

### Mode 명령어

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Mode 이름 규칙: `[a-z0-9][a-z0-9_-]*` (`mode`는 예약어).

### Channel 명령어 (선택적 브릿지)

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

`channel add`는 봇 토큰, 허용된 사용자, Profile, Mode 매핑을 안내합니다.

#### 지원 플랫폼

| 플랫폼 | 수신 방식 | 대화형 버튼 | 비고 |
|----------|-----------|-------------------|-------|
| Telegram | 롱폴링 + 웹훅 | 인라인 키보드 | 가장 완전함 |
| Slack | 이벤트 구독 웹훅 | Block Kit 액션 | HMAC-SHA256 검증 |
| Discord | 인터랙션 웹훅 | Action row 컴포넌트 | Ed25519 검증 |

#### Channel 봇 명령어

실행 중이면 봇이 채팅에서 다음 명령어에 응답합니다:

- `/help` — 사용 가능한 명령어 표시
- `/cancel` — 현재 작업 취소
- `/model` — Claude 모델 변경 (대화형 버튼)
- `/yolo` — 자동 허용 권한 토글
- `/status` — 세션 상태, Profile, Mode, git 브랜치, 토큰 사용량 표시
- `/sessions` — 최근 Claude 세션 나열 (전환 버튼 포함)
- `/projects` — 프로젝트 나열 (탐색 버튼 포함)
- `/new` — 새 세션 시작
- `/history` — 최근 세션 기록 표시

다른 텍스트를 보내면 Claude와 직접 대화합니다.

#### 권한 프롬프트

Claude가 도구 사용(명령어 실행, 파일 편집 등)에 대한 승인을 요청하면, 봇이 채팅에 대화형 허용/거부 프롬프트를 보냅니다. 버튼을 탭하면 응답이 Claude에 전달되고 처리가 자동으로 계속됩니다.

#### 시크릿

`~/.claudy/secrets.env`에 자격증명을 저장하세요:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### 에이전트 MCP 브릿지

`claudy mcp`를 실행하면 Claude Code가 다른 로컬에 설치된 AI 코딩 에이전트에 작업을 위임할 수 있는 stdio 기반 MCP 서버가 시작됩니다.

```bash
claudy mcp
```

최초 실행 시 claudy는 자동으로 `~/.claude/settings.json`에 등록됩니다. `claudy mode create <name>`으로 Mode를 생성하면 Mode의 설정 파일에도 등록됩니다. 수동 설정은 필요 없습니다.

수동으로 등록하려면 (또는 프로젝트 수준의 `.claude/settings.json`에):

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

Claude Code는 설치된 모든 에이전트를 노출하는 `ask_agent` 도구를 볼 수 있습니다.

#### 사용 예시

등록 후 Claude Code는 다음과 같이 작업을 위임할 수 있습니다:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code가 적절한 에이전트를 선택하고 프롬프트를 전달한 뒤 결과를 반환합니다. 작업 디렉터리를 지정할 수도 있습니다:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### MCP 등록 확인

```bash
# claudy가 등록되었는지 확인
cat ~/.claude/settings.json | grep -A3 claudy

# MCP 서버를 수동으로 테스트
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### 지원 에이전트 (PATH에서 자동 감지)

| 에이전트 | 바이너리 | 헤드리스 명령어 |
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

#### 커스텀 에이전트

`~/.claudy/config.yaml`에 에이전트를 추가하세요:

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

빌트인 에이전트와 동일한 키는 기본값을 재정의합니다. `args`의 `{prompt}`는 실제 작업으로 대체됩니다.

### 분석 명령어

> **참고**: 분석 기능은 아직 개발 중입니다. 토큰 수, 비용 추정치, 기타 지표가 완전히 정확하지 않을 수 있습니다. 향후 릴리스에서 개선될 예정입니다.

```bash
claudy analytics dashboard         # 로컬 분석 대시보드 열기 (Tauri 2)
claudy analytics ingest            # ~/.claude/projects/에서 세션 데이터 수집
claudy analytics ingest --full     # 모든 파일 재수집 (체크포인트 무시)
claudy analytics ingest --project my-project  # 특정 프로젝트 수집
claudy analytics recommend         # CLI에서 사용량 권장 사항 표시
claudy analytics export            # 분석 데이터 내보내기 (JSON, 기본 30일)
claudy analytics export --format csv --days 7  # 지난 7일간 CSV로 내보내기
```

분석이 추적하는 항목:

- **토큰**: 모델과 날짜별로 그룹화된 지난 30일간의 입력, 출력, 캐시 토큰 상세 추세.
- **도구**: Claude가 가장 자주 사용하는 도구를 보여주는 분포 분석(호출 횟수, 오류율, 평균 실행 시간 포함).
- **비용**: 실제 토큰 가격 기반의 실시간 사용 비용 추정(일별/주별/월별 예측 및 추세 감지 포함).
- **팁(권장 사항)**: 고비용 세션 감지, 간단한 작업에 Haiku 제안, 컨텍스트 요약이 도움될 긴 대화 식별 등 데이터 기반 최적화 조언.
- **프로젝트**: 암호화된 세션 UUID를 사람이 읽을 수 있는 프로젝트 폴더 이름으로 자동 매핑.

데이터는 `~/.claudy/analytics/` 아래 로컬 SQLite 데이터베이스에 저장됩니다. 대시보드는 고성능 로컬 Tauri 2 + Svelte 앱으로 실행됩니다. 대시보드의 **[Sync]** 버튼을 사용하여 Claude CLI 기록에서 데이터를 즉시 새로고침하세요.

<img src="../../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## 파일 및 디렉터리 구조

기본적으로 Claudy는 데이터를 다음 위치에 저장합니다:

```text
~/.claudy/
```

주요 파일/디렉터리:

- `config.yaml`: Provider + Channel + 에이전트 설정.
- `secrets.env`: Provider/봇 자격증명.
- `launchers.json`: 런처/심볼릭 링크 매니페스트.
- `modes/`: Claude 설정 Mode.
- `session-patches/`: 세션 패치 저장소.
- `channel/`: Channel 런타임 상태(`pid`, 세션, 감사 로그).
- `analytics/`: 분석 SQLite 데이터베이스 및 체크포인트.
- `cache/update.json`: 업데이트 메타데이터 캐시.

## 환경 변수

- `CLAUDY_HOME`: Claudy 홈 디렉터리 재정의 (기본값: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: Mode로 실행 시 Claudy가 자동으로 설정.

## 일반적인 워크플로우

### Provider 설정 및 실행

```bash
claudy setup
claudy <profile>
```

### Provider와 함께 Mode 사용

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo`는 `--dangerously-skip-permissions`의 claudy 단축어입니다.

### MCP를 통해 다른 에이전트에 작업 위임

```bash
# 1) MCP가 등록되었는지 확인 (첫 번째 `claudy mcp` 실행 시 자동으로 수행)
claudy mcp

# 2) Claude Code에서 설치된 에이전트에 위임 요청:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### 설치/설정 상태 진단

```bash
claudy doctor
claudy ping
```

## 문제 해결

- **`profile not recognized`**: `claudy ls`를 실행하고 나열된 Profile ID를 선택하세요.
- **`not configured` Profile**: `claudy setup <provider>`를 실행하여 자격증명을 추가하세요.
- **Channel 상태 비정상**: `claudy channel status`를 실행한 후 `claudy channel stop`과 `claudy channel start`로 재시작하세요.
- **Channel 봇 응답 없음**: 오류는 `~/.claudy/channel/logs/server.log`에서 확인하세요. `~/.claudy/secrets.env`의 봇 토큰과 `allowed_users`에 채팅 사용자 ID가 포함되어 있는지 확인하세요.
- **권한 프롬프트 미표시**: Claude CLI가 `--dangerously-skip-permissions`로 실행되지 않는지 확인하세요. 프롬프트는 Claude가 도구 사용에 대한 명시적 승인이 필요할 때만 트리거됩니다.
- **설치 후 바이너리를 찾을 수 없음**: Claudy의 bin 디렉터리가 `PATH`에 있는지 확인한 후 셸을 재시작하세요.
- **MCP에서 에이전트가 표시되지 않음**: 에이전트 바이너리가 `PATH`에 있는지 확인하세요(`which gemini`). 설치된 에이전트만 `tools/list`에 표시됩니다.
- **에이전트 타임아웃**: `config.yaml` 에이전트 필드에서 타임아웃을 늘리세요 (기본값: 120초).
- **MCP가 등록되지 않음**: `claudy mcp`를 한 번 수동으로 실행하거나 `~/.claude/settings.json`에서 `mcpServers.claudy` 항목을 확인하세요.
- **에이전트 출력 잘림**: 에이전트 stdout은 10MB로 제한됩니다. 큰 출력의 경우 에이전트가 파일에 쓰도록 리디렉션하세요.
- **분석 데이터 없음**: `claudy analytics ingest`를 실행하여 `~/.claude/projects/`에서 데이터를 채우세요. `--full`을 사용하면 모든 것을 재수집합니다.

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
2. 적절한 경우 테스트와 함께 변경 사항을 만드세요.
3. 제출 전에 `cargo test && cargo clippy -- -D warnings`를 실행하세요.
4. https://github.com/epicsagas/claudy에서 Pull Request를 여세요.

버그 리포트와 기능 요청은 [GitHub Issues](https://github.com/epicsagas/claudy/issues)를 통해 환영합니다.

## 감사의 말

이 프로젝트는 Go 기반의 Claude CLI 멀티 Provider 런처인 [Clother](https://github.com/jolehuit/clother)에서 영감을 받았습니다. Claudy는 독립적인 Rust 구현으로, 처음부터 새로 설계되었으며 RAII 기반 세션 가드, 신호 전달, 런처 심볼릭 링크, **완전한 기능의 Channel 브릿지**(Telegram/Slack/Discord), 교차 에이전트 위임을 위한 **에이전트 MCP 브릿지**, Tauri 2로 구축된 **고성능 분석 대시보드** 등 심층 생태계 통합이 도입되었습니다. 이러한 추가 기능은 Claudy가 단순한 런처에서 Claude CLI 사용자를 위한 포괄적인 운영 툴킷으로 전환했음을 반영합니다.

## 라이선스

[Apache-2.0](../../LICENSE)
