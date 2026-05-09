# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- CLI: `ls` renamed to `list` (old name works as deprecated alias)
- CLI: `mode rm` renamed to `mode remove` (old name works as deprecated alias)

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
- YAML-based configuration with automatic migration from legacy `config.json`
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
