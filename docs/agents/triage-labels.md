# Triage Labels

Default triage labels used across the project.

## Labels

### `needs-triage`

Applied automatically to all new issues via templates and Dependabot. Indicates the issue has not yet been reviewed.

### `needs-info`

The reporter must provide additional details before triage can proceed. Typically:

- Claudy version (`claudy --version`)
- Steps to reproduce
- Expected vs actual behavior
- Relevant log output or error messages

### `ready-for-agent`

A well-scoped issue suitable for an AI agent to pick up. Requirements:

- Clear acceptance criteria
- No ambiguous requirements or open design questions
- All necessary context is available in the issue body
- No security or breaking-change implications

### `ready-for-human`

Requires human judgment. Typical reasons:

- Design decisions or architecture trade-offs
- Security-sensitive changes
- Breaking changes requiring migration planning
- Stakeholder alignment needed

### `wontfix`

Acknowledged but will not be addressed. Always include a comment explaining the rationale.

## Transition Flow

```
                    ┌──────────────┐
                    │ needs-triage │  ← all new issues
                    └──────┬───────┘
                           │
              ┌────────────┼────────────┐
              │            │            │
              ▼            ▼            ▼
       ┌──────────┐ ┌────────────┐ ┌───────────────┐
       │ needs-info│ │ready-for-  │ │ ready-for-    │
       │          │ │  agent     │ │  human        │
       └────┬─────┘ └────────────┘ └───────────────┘
            │              │               │
            │              ▼               ▼
            │         (agent PR)      (human PR)
            │              │               │
            ▼              ▼               ▼
       (re-triage)    ┌──────────────────────┐
                      │      closed          │
                      └──────────────────────┘
```

After `needs-info` is resolved, the issue returns to `needs-triage` for re-evaluation.

## Applying Labels

```bash
# Triage an issue
gh issue edit 42 --remove-label "needs-triage" --add-label "ready-for-agent"

# Request more info
gh issue edit 42 --remove-label "needs-triage" --add-label "needs-info"
gh issue comment 42 --body "Could you provide the claudy version and reproduction steps?"

# Wontfix
gh issue edit 42 --remove-label "needs-triage" --add-label "wontfix"
gh issue comment 42 --body "Closing: this is expected behavior when ..."
```
