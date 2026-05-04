# AGENTS.md

## Commands
- Build: `cargo build` | Build release: `cargo build --release`
- Test: `cargo test` | Test single: `cargo test --lib <module>::tests::<fn_name>`
- Lint: `cargo clippy -- -D warnings` | Format: `cargo fmt`
- Run: `cargo run -- <subcommand>` (e.g. `cargo run -- ls`)
- Gen/Sync: N/A

## Project Structure

Architecture: **Hexagonal (ports & adapters)** — domain logic depends on ports (traits), not adapters (implementations).

```
src/
├── main.rs                    — Binary entry: collect args, call claudy::run()
├── lib.rs                     — Delegates to application::bootstrap
│
├── domain/                    — Pure business types, zero framework deps
│   ├── launch_blueprint.rs    — Target, LaunchBlueprint value objects
│   ├── commands.rs            — DomainCommand, Options
│   ├── context.rs             — Context (paths, config, secrets, catalog, output, prompt)
│   ├── channel_events.rs      — Platform, IncomingEvent, OutboundMessage, ConversationId
│   └── channel_session.rs     — ChannelSession, SessionStatus, DeliveryAttempt
│
├── ports/                     — Trait interfaces (the boundary)
│   ├── launch_ports.rs        — ProfileGateway, SecretGateway, RuntimeGateway
│   ├── config_ports.rs        — PathsPort, ConfigPort, SecretPort
│   ├── catalog_ports.rs       — CatalogPort
│   ├── command_ports.rs       — CommandGateway
│   ├── channel_ports.rs       — ChannelPort (async), SessionStore, DeliveryStore
│   └── ui_ports.rs            — OutputPort, PrompterPort
│
├── application/               — Orchestration, no knowledge of concrete types
│   ├── bootstrap.rs           — run_cli_session(): argv → shim/launcher/subcommand
│   ├── entrypoint.rs          — launch_profile_session(): wires adapters to orchestrator
│   ├── launch_orchestrator.rs — Generic 3-phase: resolve → env → spawn
│   ├── command_bus.rs         — Subcommand dispatch
│   └── channel_handler.rs     — Telegram/Slack/Discord message routing
│
└── adapters/                  — Concrete implementations of ports
    ├── cli/                   — clap arg definitions, parse, help
    ├── commands/              — Subcommand handlers (dispatch, config_cmd/*, list, info, status, test, install, mode_cmd, channel_cmd, update, uninstall)
    ├── config/                — paths (XDG), store (config.yaml), secrets (secrets.env), atomic (temp+rename)
    ├── infrastructure/        — catalog (providers JSON), models_dev, capabilities (AuthStrategy, CapabilityProfile)
    ├── launch/                — CatalogProfileAdapter, SecretEnvAdapter, RuntimeSessionAdapter
    ├── profiles/              — resolve (catalog → or-* → custom chain)
    ├── runtime/               — env (EnvBuilder), overlay/pipeline (provider env injection), exec (launch), claude (binary discovery), args
    ├── channel/               — server (axum), sessions, commands, stream_handler, audit, pid, retry, idempotency, service managers
    │   ├── telegram/          — api, normalize, buttons
    │   ├── slack/             — api, normalize, blocks, webhook
    │   └── discord/           — api, normalize, components, webhook
    ├── ui/                    — output (human/json/plain), prompt
    ├── update/                — check (VersionProvider trait), install, states
    └── version.rs
```

## Code Style
- Rust 2024 edition, MSRV 1.92
- Naming: `snake_case` functions/variables, `PascalCase` types, `UPPER_SNAKE` constants
- Error handling: `anyhow::Result<T>` everywhere; `thiserror` for domain error types; never panic in library code
- Async: CLI core is synchronous; channel subsystem uses `tokio` + `async_trait` + `axum`
- File writes: Always via `adapters::config::atomic::write_atomic()` — never `std::fs::write` for user data
- Environment variables: `CLAUDY_*` for claudy-specific, `ANTHROPIC_*` for Claude CLI integration
- Dependency rule: `domain` → `ports` ← `adapters`; `application` depends on `ports` only, never `adapters` directly
- Unix-only: `std::os::unix::fs::symlink`, `libc::signal` for process management
- Provider capability: Use `CapabilityProfile` trait (`infrastructure/capabilities.rs`) for auth/model-tier decisions, not string matching

### Golden Path
```rust
// Launch orchestrator pattern — generic over port traits
pub struct LaunchOrchestrator<P, S, R> {
    profile_gateway: P,
    secret_gateway: S,
    runtime_gateway: R,
}

impl<P: ProfileGateway, S: SecretGateway, R: RuntimeGateway> LaunchOrchestrator<P, S, R> {
    pub fn dispatch(self, blueprint: LaunchBlueprint) -> anyhow::Result<i32> {
        let target = self.profile_gateway.resolve_target(&blueprint.profile)?;
        let env = self.secret_gateway.build_provider_env(&target)?;
        self.runtime_gateway.run_target(&target, &blueprint.forwarded_args, &env)
    }
}
```

## Testing
- Framework: Built-in `#[test]` + `cargo test`
- Run all: `cargo test` | Coverage: N/A
- File naming: Inline `#[cfg(test)] mod tests` in same file
- Mocking: `tempfile::tempdir()` for filesystem; `serial_test` for env-var tests; port traits enable full adapter mocking
- Test count: ~154 test functions across 37 test modules

## Git Workflow
- Branch strategy: Direct push to `main` for small changes; feature branches for large changes
- Commit format: Conventional Commits (`type(scope): description`)
- Release: `cargo-dist` builds cross-platform binaries (macOS/Linux/Windows)

## Boundaries
- Always: Depend on ports (traits), not adapters (concrete types) in `domain/` and `application/`
- Always: Use `config::atomic::write_atomic()` for file writes that must not corrupt on crash
- Always: Use `CapabilityProfile` trait for auth/model-tier decisions — not raw string matching on `family`
- Always: Include user-actionable guidance in error messages
- Always: Run `cargo clippy -- -D warnings` and `cargo fmt` before committing
- Ask first: Adding new ports (affects domain/application contract)
- Ask first: Changing adapter/port interface signatures (ripple effect across layers)
- Ask first: Modifying session patch/restore logic (data safety critical)
- Never: Import from `adapters::*` in `domain/` or `application/` — use `ports::*` instead
- Never: Use `std::fs::write()` for user-facing files — always atomic write
- Never: Use `unwrap()` in non-test code — use `?`, `ok_or_else()`, or explicit handling
- Never: Use `#[allow(...)]` to suppress warnings — fix the root cause; use `#[expect(...)]` only with a tracking issue comment

## Agent skills

### Issue tracker

Issues tracked in GitHub (epicsagas/claudy). Uses `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default triage labels: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout. See `docs/agents/domain.md`.
