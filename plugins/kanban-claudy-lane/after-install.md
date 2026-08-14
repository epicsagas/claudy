# kanban-claudy-lane installed

The `kanban-claudy-lane` skill is now available. Two things to finish setup:

## 1. Verify claudy is installed

The lane delegates implementation to claudy (the Claude CLI launcher):

```bash
command -v claudy && claudy --version
```

claudy runs `claudy -p "<prompt>" --yolo` in its cwd (`--yolo` → `--dangerously-skip-permissions`). If claudy is not installed, the lane gracefully skips (`claudy_lane.used: false`) and the worker does the work directly — but you lose the iteration-budget savings.

## 2. Point your dev profile at it

The lane only fires when a worker profile loads the skill. Add this block to your implementation profile's `SOUL.md` (e.g. `~/.hermes/profiles/dev-lead/SOUL.md`):

```markdown
## Implementation Delegation (claudy lane)
- For implementation tasks (DEV-*), prefer delegating via `kanban-claudy-lane`
  before spending the full iteration budget on direct coding.
- This profile owns worktree creation, review, test re-run, and handoff.
  claudy is an input lane only.
- Invocation: `claudy -p "<prompt>" --yolo` with cwd = isolated git worktree.
- Skip when a direct edit is smaller/safer or criteria are unclear.
```

Then restart your gateway (`hermes gateway restart`) so the profile picks up the skill.
