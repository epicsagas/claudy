# config.yaml 레퍼런스

모든 설정은 `~/.claudy/config.yaml`에 저장됩니다. 필요한 섹션만 추가하면 됩니다 — 생략된 항목은 기본값이 적용됩니다.

- [최상위 구조](#최상위-구조)
- [provider\_overrides](#provider_overrides)
- [openrouter\_aliases](#openrouter_aliases)
- [custom\_providers](#custom_providers)
- [compaction](#compaction)
- [model\_settings](#model_settings)
- [channel](#channel)
- [agents](#agents)
- [전체 예시](#전체-예시)

---

## 최상위 구조

| 필드 | 타입 | 기본값 | 설명 |
|---|---|---|---|
| `version` | `int` | `1` | 스키마 버전. 향후 마이그레이션을 위해 예약됨. |
| `provider_overrides` | `map<string, ModelPreset>` | `{}` | 내장 프로바이더별 기본 모델/티어 재정의. |
| `openrouter_aliases` | `map<string, string>` | `{}` | 단축명 → OpenRouter 모델 ID 매핑. |
| `custom_providers` | `map<string, UserEndpoint>` | `{}` | Anthropic 호환 서드파티 엔드포인트. |
| `compaction` | `ContextWindowPolicy` | 아래 참조 | 컨텍스트 자동 압축 동작. |
| `model_settings` | `map<string, PerModelOverrides>` | `{}` | 모델별 컨텍스트 윈도우 재정의. |
| `channel` | `BridgeSettings` | 아래 참조 | 채널 브릿지 설정 (Telegram/Slack/Discord). |
| `agents` | `map<string, AgentConfig>` | `{}` | 에이전트 재정의 / 커스텀 에이전트. |

---

## provider_overrides

내장 프로바이더(예: `zai`, `anthropic`, `ollama`)의 기본 모델과 모델 티어를 재정의합니다.

| 필드 | 타입 | 기본값 | 설명 |
|---|---|---|---|
| `model` | `string` | `""` | 해당 프로바이더에서 사용할 기본 모델. 모델 ID 문자열 또는 `model_choices`의 1-기반 인덱스를 사용할 수 있습니다. |
| `model_tiers` | `map<string, string>` | `{}` | 티어명 → 모델 ID 매핑. 지원 티어: `opus`, `sonnet`, `haiku`, `small`. `ANTHROPIC_DEFAULT_<TIER>_MODEL`을 설정합니다. |

```yaml
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"
      sonnet: "glm-5.1"
      opus: "glm-5"
```

---

## openrouter_aliases

단축명을 OpenRouter 모델 ID에 매핑합니다. `claudy or <alias>` 로 실행합니다.

```yaml
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"
  gemini: "google/gemini-2.5-pro"
```

---

## custom_providers

Anthropic 호환 서드파티 엔드포인트를 등록합니다. `claudy <slug>` 로 실행합니다.

| 필드 | 타입 | 필수 | 설명 |
|---|---|---|---|
| `name` | `string` | 예 | 내부 식별자 (맵 키와 동일). |
| `display_name` | `string` | 예 | `claudy ls`에 표시되는 이름. |
| `base_url` | `string` | 예 | 엔드포인트 기본 URL (Anthropic API 호환이어야 함). |
| `api_key_env` | `string` | 예 | API 키가 저장된 환경 변수명 (예: `MY_LLM_API_KEY`). |
| `default_model` | `string` | 아니오 | 이 프로바이더의 기본 모델 ID. |

```yaml
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.example.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"
```

---

## compaction

Claude의 컨텍스트 윈도우 압축 타이밍을 제어합니다.

| 필드 | 타입 | 기본값 | 설명 |
|---|---|---|---|
| `auto_compact` | `bool` | `true` | 컨텍스트가 임계값에 도달할 때 자동 압축 활성화. |
| `threshold` | `float` | `0.8` | 압축을 트리거할 컨텍스트 윈도우 사용 비율 (0.0–1.0). |

```yaml
compaction:
  auto_compact: true
  threshold: 0.85
```

---

## model_settings

모델별 컨텍스트 윈도우 재정의. 키는 모델 ID 문자열입니다.

| 필드 | 타입 | 설명 |
|---|---|---|
| `max_context_tokens` | `uint` | 해당 모델의 컨텍스트 토큰 상한선. |
| `compaction_threshold` | `float` | 모델별 압축 임계값 (0.0–1.0). `compaction.threshold`보다 우선합니다. |

```yaml
model_settings:
  deepseek-chat:
    max_context_tokens: 64000
  claude-opus-4-5:
    compaction_threshold: 0.9
```

---

## channel

Telegram, Slack, Discord 채널 브릿지를 설정합니다.  
`claudy channel add`의 비대화형 대안입니다.

### 기본 설정

| 필드 | 타입 | 기본값 | 설명 |
|---|---|---|---|
| `enabled_platforms` | `string[]` | `[]` | 활성화할 플랫폼: `telegram`, `slack`, `discord`. |
| `listen_addr` | `string` | `"127.0.0.1:3456"` | 브릿지 HTTP 서버가 바인딩할 주소:포트. |
| `stream_timeout_secs` | `uint` | `1800` | Claude 응답 스트림 최대 대기 시간 (초). |
| `max_concurrent_sessions` | `uint` | `0` | 최대 동시 Claude 세션 수. `0` = 무제한. |

### 프로바이더 라우팅

| 필드 | 타입 | 설명 |
|---|---|---|
| `default_profile` | `string` | 플랫폼 레벨 재정의가 없을 때 사용할 프로바이더 프로파일. |
| `platform_profiles` | `map<string, string>` | 플랫폼별 프로파일. 키 = 플랫폼명 (`telegram`/`slack`/`discord`). |
| `channel_profiles` | `map<string, string>` | 채널별 프로파일. 키 = `"platform:channel_id"` 또는 `"platform:guild_id:channel_id"` (Discord). |

조회 순서: `channel_profiles["platform:guild_id:channel_id"]` → `channel_profiles["platform:guild_id"]` → `channel_profiles["platform:channel_id"]` → `platform_profiles["platform"]` → `default_profile`.

### 모드 라우팅

| 필드 | 타입 | 설명 |
|---|---|---|
| `default_mode` | `string` | 모든 플랫폼에 적용될 기본 모드명. |
| `platform_modes` | `map<string, string>` | 플랫폼별 모드. 키 = 플랫폼명. |
| `channel_modes` | `map<string, string>` | 채널별 모드. 키 형식은 `channel_profiles`와 동일. |

### 프로젝트 라우팅

| 필드 | 타입 | 설명 |
|---|---|---|
| `default_project` | `string` | 기본 작업 디렉토리 절대 경로. |
| `channel_projects` | `map<string, string>` | 채널별 프로젝트 디렉토리. 키 형식은 `channel_profiles`와 동일. |

### 접근 제어

| 필드 | 타입 | 설명 |
|---|---|---|
| `allowed_users` | `string[]` | 모든 플랫폼에서 허용할 사용자 ID / 사용자명. 비어 있으면 모두 허용. |
| `platform_allowed_users` | `map<string, string[]>` | 플랫폼별 `allowed_users` 재정의. 키 = 플랫폼명. |

```yaml
channel:
  enabled_platforms: ["telegram", "discord"]
  listen_addr: "127.0.0.1:3456"
  stream_timeout_secs: 1800
  max_concurrent_sessions: 4

  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
    discord: "anthropic"
  channel_profiles:
    "discord:guild123:channel456": "openrouter"

  default_mode: "default"
  platform_modes:
    telegram: "focus"

  default_project: "/home/user/projects/main"
  channel_projects:
    "telegram:987654321": "/home/user/projects/side"

  allowed_users: ["user_id_1", "user_id_2"]
  platform_allowed_users:
    discord: ["discord_user_id_3"]
```

---

## agents

내장 에이전트 기본값을 재정의하거나 커스텀 에이전트를 등록합니다.  
내장 에이전트명: `codex`, `copilot`, `agent`, `opencode`, `cline`, `goose`, `amp`, `droid`, `kiro`, `junie`, `kimi`, `vibe`, `qwen-code`, `crush`, `groq-code`, `plandex`, `kilo`, `openhands`.

모든 필드는 선택 사항입니다. 내장 에이전트의 경우 명시된 필드만 기본값을 덮어씁니다.  
**커스텀 에이전트** (내장 목록에 없는 키)의 경우 `binary`는 필수입니다.

| 필드 | 타입 | 기본값 | 설명 |
|---|---|---|---|
| `binary` | `string` | 내장 기본값 | 실행 파일명 또는 절대 경로. 커스텀 에이전트는 필수. |
| `args` | `string[]` | 내장 기본값 | 인수 목록. `{prompt}`는 실제 태스크 문자열로 치환됩니다. |
| `description` | `string` | 내장 기본값 | `claudy mcp list-agents`에 표시되는 설명. |
| `timeout` | `uint` | 내장 기본값 | 실행 타임아웃 (초). `CLAUDY_AGENT_TIMEOUT` 환경 변수로도 설정 가능. |

타임아웃 우선순위: `agents.<name>.timeout` > `CLAUDY_AGENT_TIMEOUT` 환경 변수 > 내장 기본값.

```yaml
agents:
  # timeout만 재정의 — binary와 args는 내장 기본값 유지
  codex:
    timeout: 7200

  # 내장 에이전트 전체 재정의
  aider:
    binary: "aider"
    args: ["--message", "{prompt}", "--yes-always"]
    timeout: 600

  # 커스텀 에이전트 등록 (binary 필수)
  my-agent:
    binary: "/usr/local/bin/my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "나의 커스텀 코딩 에이전트"
    timeout: 300
```

---

## 전체 예시

```yaml
version: 1

provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"
      sonnet: "glm-5.1"
      opus: "glm-5"

openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.example.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

compaction:
  auto_compact: true
  threshold: 0.85

model_settings:
  deepseek-chat:
    max_context_tokens: 64000

channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0
  stream_timeout_secs: 1800

agents:
  codex:
    timeout: 7200
  my-agent:
    binary: "/usr/local/bin/my-agent"
    args: ["--prompt", "{prompt}"]
    description: "나의 커스텀 코딩 에이전트"
    timeout: 300
```
