# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-07-21

### Added

- **Analytics ingestion reliability** — session ingestion is now self-scheduling, has a durable archive fallback, and exposes a freshness check. This fixes a silent ~7.5-week ingestion freeze (2026-05-28 → 2026-07-20) where the DB stopped advancing while the file mtime stayed current (pricing-sync kept it fresh). (#52)
  - `claudy analytics schedule {install,uninstall,status}` — periodic ingestion via a macOS LaunchAgent (`com.claudy.analytics.ingest`, `StartInterval`/`RunAtLoad`) or a Linux `--user` systemd timer, modeled on the existing service-install seams but kept a periodic one-shot (not a daemon). Mtime checkpoints make hourly re-runs cheap and idempotent.
  - `claudy analytics status [--stale-days N] [--json]` — reports the last-recorded session time and per-source last-seen; exits non-zero when data is older than `--stale-days` (an empty DB is not "stale"). This is the alarm that would have caught the freeze on day one.
  - Archive fallback: `[analytics]` config (`sources`, `archive_root`, `archive_on_ingest`, all defaulted on) mirrors live JSONL into `~/.claude/projects-archive` during ingest (symlink-safe, `0o600`) and ingests archive sources as gap-fillers only, de-duplicated by `session_uuid`. New neutral `source_kind` column on `sessions` tags each row's origin.
  - `human_authored` flag on `turns` (derived: `!is_meta && !is_command`) and best-effort backfill of turns whose model is NULL from the session model. Code-authorship-level human-vs-AI classification (needs git) is intentionally out of scope for claudy and left to downstream consumers.
  - Schema migrations: v1 (idempotent `ALTER TABLE`) adds `sessions.source_kind` and `turns.human_authored`; v2 dedupes existing turns and adds `UNIQUE(session_id, turn_number)` (plus a new-turn insert gate), so the hourly scheduler can no longer compound-duplicate turns, token-usage, or tool-calls on an actively-growing file.
- Four previously-stubbed aggregation metrics implemented and exposed via the insights path. (`de5333f`)

### Fixed

- Incremental re-ingestion no longer re-parses a live session from line 0 on every scheduler run: `parse_and_ingest` now resumes from a persisted `byte_offset`, building on the `UNIQUE(session_id, turn_number)` dedup gate above (#53). Session-level `total_cost_usd` / `total_duration_ms` are now preserved across that incremental resume — previously an appended-only parse reset them to just the appended portion's totals (#54). Regression tests cover gap-fill/dedup, incremental append, partial trailing-line re-read, and shrunk-file offset clamping.
- Borrow-check and dead-code nits in the Linux schedule-unit generation and the macOS-only `build_service_path`.

## [0.4.0] - 2026-07-16

### Added

- Opt-in shell-environment loading for spawned processes. When `CLAUDY_SHELL_ENV=1` (or `true`), the login-shell environment (`$SHELL -l -c 'env -0'`) is merged into the Claude CLI / MCP agent runner processes, so PATH additions and exports defined only in `~/.zshrc` / `~/.bash_profile` / `~/.profile` are visible to claudy even when it was launched from a non-login context (GUI, launchd, IDE task). Existing process values always win (merge is `entry().or_insert()`); off by default. (#41, #44)

### Fixed

- Symlinked project directories are now resolved before launching Claude. A configured project dir (channel `channel_projects`, `default_project`, or platform override) that points through a symlink is canonicalized via `dunce::canonicalize` in both the channel launch path and the MCP agent runner, so dotfile/vault-managed layouts land at their real target. Falls back to the raw path on canonicalization failure so the existing `is_dir()` guard is never hardened into a new error path. (#40, #43)

### Changed

- Upgrade `llm-kernel` 0.10 → 0.20 and adapt to the new `McpServer::initialize_response(requested_version: Option<&str>)` signature — the runtime `initialize` handler now passes the client's requested `protocolVersion` through for proper MCP version negotiation. Aligns claudy with the latest llm-kernel, resolving cross-project version fragmentation. (#42)
- Bump `plist` 1.9.0 → 1.10.0 to clear `quick-xml` RUSTSEC-2026-0194 / RUSTSEC-2026-0195 (high, 7.5 — quadratic duplicate-attribute check + unbounded namespace-declaration allocation). `quick-xml` is pulled transitively via `plist` → `tauri`.
- Add `dunce` as a direct dependency (already transitively present via `tauri`) for Windows-safe symlink canonicalization.
- Dependency bumps: `anyhow` 1.0.102 → 1.0.103, `uuid` 1.23.3 → 1.23.4, `which` 8.0.3 → 8.0.5, `serial_test` 3.4.0 → 3.5.0, `tower-http` 0.6 → 0.7, `tokio-tungstenite` 0.29 → 0.30, `ed25519-dalek` 2 → 3, and `dtolnay/rust-toolchain` GitHub Action. (#34, #36, #37, #38, #39)

### Verified

- Confirmed end-to-end that cross-provider resume (ZAI/GLM → Claude `--resume`) works after `tool_use` id sanitization; `call_<hex>` ids are rewritten to `toolu_*` before resume with no HTTP 400. (#33)
- Confirmed the stream-json `is_error: true` gate fires correctly on real 529 (overloaded) events from the z.ai gateway — the transient-API recovery path runs to completion (3 backoff retries → depth exceeded → user notified), not a silent fail. (#32)

## [0.3.11] - 2026-06-29

### Fixed

- Channel sessions no longer misclassify a normal assistant response that merely mentions "rate limit" / "overloaded" / "try again later" as a transient 529 API error. Transient-API recovery is now gated on the stream-json `is_error: true` flag (`classify_transient_api_error`), so the real response is never discarded and replayed 3×. The previous tautological condition that made the `is_error` branch dead code is removed. (#31)
- Transient-API (529/429/503) recovery no longer consumes the context-limit compaction budget. The two recovery paths now use independent depth counters (`TRANSIENT_RECOVERY_DEPTH` vs `RECOVERY_DEPTH`), so a single transient retry can no longer trip the context-limit guard and clear the session instead of compacting. (#31)
- Recovery re-entry (transient and context-limit) now kills the tracked Claude PID instead of merely untracking it, preventing a detached process from lingering and writing to the same session JSONL while the replay spawns a new one. (#31)

### Changed

- `sanitize_session` reads and parses the session JSONL once and threads the content through all sanitizer cores in memory, instead of re-reading/re-parsing the (potentially large) file for every sanitizer on each resume. (#31)
- Dedupe the two transient-recovery entry blocks into a shared `enter_transient_recovery` helper, and unify the `toolu_` / `srvtoolu_` id validators under `is_valid_prefixed_id`. (#31)

## [0.3.8] - 2026-06-16

### Fixed

- Drop the unsupported `--output-format text` flag from the `agy` (Antigravity) agent mapping. agy 1.0.x has no such flag, so every `ask_agent` delegation to agy failed with `flags provided but not defined: -output-format`; agy `--print` emits plain text by default, so the flag was unnecessary. Updated the builtin definition (`agent.rs`) and the README headless-command column across all translations. The Cursor `agent` CLI (which does support the flag) is unchanged.

## [0.3.7] - 2026-06-16

### Fixed

- Emit `inputSchema` (camelCase) in the MCP `tools/list` response so strict MCP 2024-11-05 clients (e.g. Claude Code) accept the `ask_agent` tool instead of rejecting it with `tools[0].inputSchema: expected object, received undefined`. `llm-kernel`'s `ToolDescription` serializes the field as `input_schema` (snake_case); the server now emits each tool explicitly with the spec key.

### Changed

- Cross-link the `epic-harness` synergy across the README and all 10 i18n translations

## [0.3.6] - 2026-06-13

### Added

- Integrate `llm-kernel` for provider catalog, secrets vault, and MCP protocol (#17)
- `status` warns when an external `CLAUDE_CODE_BLOCKING_LIMIT_OVERRIDE` would cap the context window below a model's configured `max_context_tokens`

### Fixed

- Channel/headless sessions now inject model compaction env vars (`CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`), matching the interactive launch path
- Selecting a project in a channel (Telegram/Discord) now persists it as the new session's working directory, so "New → Current project" launches in that project's context
- Improve vault type safety, observability, and test coverage
- Pin `llm-kernel` to 0.3.6 and commit `Cargo.lock` so CI and local builds resolve identical dependency versions (the lock was gitignored, letting CI pull a newer `llm-kernel` whose `load_from`/`persist_to` return `KernelError` instead of `anyhow::Error`, breaking the build)

### Changed

- Bump `actions/checkout` from 6.0.2 to 6.0.3

## [0.3.4] - 2026-06-03

### Fixed

- Replace unrecognized `ANTHROPIC_CONFIG_OVERRIDE` with Claude Code's actual compaction env vars (`CLAUDE_AUTOCOMPACT_PCT_OVERRIDE`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`) so auto-compaction works with non-native providers
- Apply safe 60% compaction fallback when `auto_compact: false` to prevent context overflow in autonomous sessions

### Changed

- Update rusqlite requirement from 0.39 to 0.40
- Update gitleaks/gitleaks-action from 2.3.9 to 3.0.0

## [0.3.3] - 2026-05-29

### Added

- Channel bridge: context limit recovery and `/compact` command fix

### Changed

- Remove gemini-cli from builtin agents; replace with antigravity in docs

## [0.3.2] - 2026-05-20

### Fixed

- Remove aarch64-linux target due to ports.ubuntu.com outage

## [0.3.1] - 2026-05-20

### Added

- Shell and PowerShell one-liner installers via cargo-dist

### Changed

- Make publish-crates CI job non-blocking
- Update GitHub Actions to latest stable versions
- Bump Swatinem/rust-cache and gitleaks/gitleaks-action

## [0.3.0] - 2026-05-16

### Added

- `claudy session sanitize` command — finds sessions with invalid thinking blocks written by non-Anthropic providers (e.g. Z.AI / GLM) and converts them to text blocks so the session can be resumed with the Anthropic API without a `400 Invalid signature in thinking block` error. Supports `--project` filter, `--all` flag, and `-y` to skip confirmation.
- Channel bridge: automatic session sanitization before `--resume` — when the channel server resumes a session that contains thinking blocks with empty signatures, it silently converts them before spawning the Claude process, making provider switching transparent.
- Channel bridge: stderr detection for `Invalid signature in thinking block` — if sanitization is bypassed or another signature error occurs at runtime, the channel server clears the session ID so the next message starts cleanly.

### Changed

- CLI: `ls` renamed to `list` (old name works as deprecated alias)
- CLI: `mode rm` renamed to `mode remove` (old name works as deprecated alias)

## [0.2.3] - 2026-05-13

### Fixed

- Remove unsupported `extra-artifacts` from dist-workspace.toml
- Correct archive filename and extraction path in install

### Added

- One-liner installer scripts with SHA-256 verification

### Changed

- Remove aider from builtin agents

## [0.2.2] - 2026-05-12

### Fixed

- Remove musl targets that fail due to tauri GTK dependency
- Continue-on-error for homebrew publish (HOMEBREW_TAP_TOKEN optional)
- Replace CODESIGN_* with APPLE_* secrets for macOS signing

### Added

- `cargo-binstall` metadata for pre-built binary support
- Linux musl targets and optimized release build
- macOS code signing and notarization

### Changed

- Sync all translated READMEs with English source
- Restructure installation and update sections in README

## [0.2.1] - 2026-05-10

### Fixed

- Add Windows-compatible `is_pid_alive` to unblock cross-platform release
- Correct Discord components payload structure causing 50035
- Resolve three root causes of unresponsive channel bridge (#7)
- Seed bundled skills during install and add missing agent docs

### Changed

- CLI: `ls` renamed to `list` (old name works as deprecated alias)
- CLI: `mode rm` renamed to `mode remove` (old name works as deprecated alias)
- Promote agent bridge and analytics to top-level README sections

## [0.2.0] - 2026-05-07

### Added

- Multi-provider launch with built-in, Z.AI, OpenRouter, Ollama, and custom endpoint support
- Config modes for isolated Claude configuration per context (`claudy mode create/ls/rm`)
- Channel bridge for Telegram (long-polling + webhook), Slack (Event subscription), and Discord (Gateway + Interaction webhook)
- Interactive permission prompts via chat buttons (Allow/Deny) for tool approval
- Agent MCP bridge to delegate tasks from Claude Code to Gemini, Codex, Aider, and 20+ other agents
- Usage analytics: token tracking, cost estimation, tool usage analysis, model distribution
- Analytics dashboard built with Tauri 2 + Svelte
- `claudy analytics insights` command for LLM-powered usage analysis
- `claudy analytics sync-pricing` for runtime pricing sync from models.dev + Anthropic
- `claudy analytics recommend` for data-driven optimization advice
- `claudy analytics export` (JSON/CSV) for data portability
- `/analytics-insights` skill integration inside Claude Code
- Per-channel project binding and per-platform user allowlists
- Discord slash commands and Telegram group chat support
- OpenRouter alias provider support
- Custom provider support (user-defined Anthropic-compatible endpoints)
- Provider credential management via `secrets.env` (0600 permissions)
- `claudy doctor` for configuration health diagnostics
- `claudy ping` for provider connectivity testing
- `claudy sync` (alias: `install`) for binary installation
- `claudy update` for self-update
- `claudy uninstall` for clean removal
- Homebrew tap installer (`brew tap epicsagas/tap && brew install claudy`)
- One-liner shell and PowerShell installers via cargo-dist
- `cargo binstall claudy` pre-built binary support
- crates.io automatic publishing via CI
- Apple Developer ID signing for macOS binaries
- Multilingual README (10 languages: Korean, Chinese, Japanese, German, French, Spanish, Hindi, Portuguese, Indonesian, Arabic)
- CONTRIBUTING.md with PR checklist and Conventional Commits guide
- FUNDING.yml (GitHub Sponsors + Buy Me a Coffee)

### Changed

- Redesigned analytics dashboard with financial-platform theme, light/dark modes, and settings panel
- Redesigned onboarding banner to display selected mode
- Improved channel interaction handling and callback routing
- Deduplicated bot command definitions across platforms
- Decomposed god objects into domain-specific submodules (hexagonal architecture)
- Extracted shared helpers to eliminate code duplication
- Switched from `config.json` to `config.yaml` format

### Fixed

- Decoupled `bin_dir` from Claude binary path and synced mode MCP registrations
- Corrected multi-provider pricing, tool error tracking, and cache savings calculation
- Fixed Claude process tracking to be per-scope instead of global
- Fixed channel interaction hardening and callback routing edge cases
- Eliminated per-request client allocation in channel handlers
- Resolved security, performance, and correctness warnings in pricing sync
- Fixed path handling in analytics skill seeding
- Fixed clippy warnings across analytics, channel, and launcher modules
- Skipped providers that need no configuration in interactive setup
- Consolidated dist config and fixed stale wix URL

### Security

- Stored all credentials in `secrets.env` with `0600` file permissions
- HMAC-SHA256 verification for Slack webhooks
- Ed25519 signature verification for Discord interactions
- Atomic file writes for all user-facing configuration changes
