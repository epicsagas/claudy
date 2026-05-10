# Domain Docs

Single-context layout — each domain file is a self-contained bounded context with no cross-domain file dependencies.

## Layout

```
src/domain/
├── launch_blueprint.rs   — Launch target and blueprint value objects
├── commands.rs           — DomainCommand and Options
├── context.rs            — Context (paths, config, secrets, catalog, output, prompt)
├── agent.rs              — AgentDefinition, AgentConfig, builtin_agents()
├── channel_events.rs     — Platform, IncomingEvent, OutboundMessage, ConversationId
├── channel_session.rs    — ChannelSession, SessionStatus, DeliveryAttempt
└── analytics.rs          — Analytics event types and aggregation models
```

## Module Responsibilities

### `launch_blueprint.rs`

Defines `LaunchTarget` (resolved profile with provider/model/env) and `LaunchBlueprint` (the user's intent: profile name, forwarded args). These are value objects — no behavior, just data.

### `commands.rs`

`DomainCommand` enum and `Options` struct representing parsed CLI input. Pure data, no I/O.

### `context.rs`

`Context` struct assembled during bootstrap. Carries all runtime dependencies: `AppPaths`, `AppRegistry`, secrets, catalog, output adapter, prompt adapter. This is the "world" passed to command handlers.

### `agent.rs`

`AgentDefinition` (resolved, ready-to-execute) and `AgentConfig` (user-configurable override from `config.yaml`). `builtin_agents()` returns the hardcoded catalog of 20 known agent CLIs.

### `channel_events.rs`

Platform-agnostic channel types: `Platform` enum (Telegram/Slack/Discord), `IncomingEvent`, `OutboundMessage`, `ConversationId`. No platform-specific logic.

### `channel_session.rs`

Session lifecycle types: `ChannelSession`, `SessionStatus`, `DeliveryAttempt`. Used by the channel handler and session store.

### `analytics.rs`

Analytics event types and aggregation models for usage tracking.

## Dependency Rules

```
domain/ → ports/  (domain depends on port traits for type signatures only)
domain/ ← application/ ← adapters/  (layers depend inward, never outward)
```

- **Never** import from `adapters::*` in `domain/` or `application/`
- **Never** import from `application::*` in `domain/`
- Each domain file is a single context — no `mod X;` imports between domain files except through `domain/mod.rs` re-exports
- Domain types are `Debug`, `Clone`, `Serialize`, `Deserialize` where appropriate — no framework-specific derives
