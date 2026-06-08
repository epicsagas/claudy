# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
