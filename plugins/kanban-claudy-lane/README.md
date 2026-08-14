# kanban-claudy-lane

A **Hermes Agent** plugin that lets a Kanban worker delegate bounded implementation work to **claudy** (the Claude CLI launcher) in an isolated git worktree, while Hermes keeps full ownership of the task lifecycle, reconciliation, testing, and handoff.

It is the **Codex-substitute lane**: same ownership model as `kanban-codex-lane`, but it shells out to `claudy -p "..." --yolo` instead of `codex`, for setups without a Codex subscription. claudy runs the Claude CLI in its cwd; isolation is achieved by pointing it at a dedicated worktree.

## What it does

A Hermes Kanban worker (e.g. the `dev-lead` profile) running a `DEV-*` task can, instead of spending all of its iteration budget on raw coding, delegate the implementation:

1. **Isolate** — create a `git worktree` + branch tied to the task id.
2. **Delegate** — run `claudy -p "<prompt>" --yolo` with the worktree as cwd. The prompt states scope, ownership rules, safety constraints, and verification commands.
3. **Reconcile** — Hermes reviews the diff, rejects anything unsafe or out of scope, and runs the canonical test suite itself.
4. **Hand off** — `kanban_complete` with a `metadata.claudy_lane` record (worktree, accepted commits, tests run by each side).

This spreads implementation iterations across claudy and keeps the Hermes worker's budget for review + verification.

## Requirements

- Hermes Agent with a Kanban board (the dispatcher + `kanban_show`/`kanban_complete` tools).
- `claudy` installed and synced (`command -v claudy`). claudy launches the Claude CLI; `--yolo` maps to `--dangerously-skip-permissions`.
- A repo the worker can check out into a git worktree.

## Install

This plugin ships inside the [claudy](https://github.com/epicsagas/claudy) repository at `plugins/kanban-claudy-lane/`. The claudy repo root is a Tauri app, so install from the **plugin subdirectory**, not the repo root.

`hermes plugins install` resolves a Git URL or `owner/repo[/subdir]` — it does **not** accept a bare local path. Use one of:

### Option A — from GitHub (after the plugin dir is pushed)

```bash
hermes plugins install epicsagas/claudy/plugins/kanban-claudy-lane --enable
# owner/repo/subdir shorthand: clones claudy, checks out the subdir only.
```

### Option B — local clone, manual copy (works before push / offline)

```bash
git clone https://github.com/epicsagas/claudy.git /tmp/claudy
cp -r /tmp/claudy/plugins/kanban-claudy-lane ~/.hermes/plugins/
hermes plugins enable kanban-claudy-lane
```

### Option C — local git repo via `file://` (after committing the dir locally)

```bash
hermes plugins install "file:///path/to/claudy#plugins/kanban-claudy-lane" --enable
```

Verify after install:

```bash
hermes plugins list
hermes plugins doctor kanban-claudy-lane   # → "OK: registration passed"
command -v claudy && claudy --version
```

## Post-install: teach your dev profile to use it

The lane is a skill — it only fires when a worker profile loads it. Add this to your implementation profile's `SOUL.md` (e.g. `profiles/dev-lead/SOUL.md`):

```markdown
## Implementation Delegation (claudy lane)
- For implementation tasks (DEV-*), prefer delegating via the `kanban-claudy-lane`
  skill before spending the full iteration budget on direct coding.
- This profile owns worktree creation, review, test re-run, and `kanban_complete`
  handoff. claudy is an input lane only.
- Invocation: `claudy -p "<prompt>" --yolo` with cwd = isolated git worktree.
- Skip when a direct edit is smaller/safer, or when acceptance criteria are unclear.
```

## Usage

A worker invokes the lane by following the skill (`skills/kanban-claudy-lane/SKILL.md`). Typical flow:

```bash
# 1. Isolate
git -C "$REPO" worktree add -b "claudy/$TASK_ID/$(date -u +%Y%m%d%H%M%S)" \
  "/tmp/$TASK_ID-claudy-lane" "$BASE"

# 2. Delegate — claudy runs in cwd (the worktree); no path argument.
claudy -p "$(cat /tmp/claudy-lane-prompt.md)" --yolo

# 3. Hermes reconciles: git diff review + canonical test re-run
# 4. kanban_complete with metadata.claudy_lane = {used, result, ...}
```

Launch via the Hermes `terminal` tool with the worktree as `workdir` so claudy scopes its work there:

```python
terminal(
    command='claudy -p "$(cat /tmp/claudy-lane-prompt.md)" --yolo',
    workdir="/tmp/<task>-claudy-lane",
    background=True, pty=True, notify_on_complete=True,
)
```

See `skills/kanban-claudy-lane/templates/claudy-lane-prompt.md` for the prompt template (fill `[BRACKETED]` fields + `[SAFETY_CONSTRAINTS]`).

## Metadata schema

Every task that considers the lane records `metadata.claudy_lane`:

```json
{
  "claudy_lane": {
    "used": true,
    "worktree": "/tmp/<task>-claudy-lane",
    "branch": "claudy/<task>/<ts>",
    "command": "claudy -p ... --yolo",
    "result": "accepted | rejected | partial | timed_out",
    "accepted_commits": ["<sha>"],
    "tests_run": [{"command": "...", "exit_code": 0, "owner": "hermes|claudy"}],
    "artifacts": ["/path/to/log"]
  }
}
```

## License

Apache-2.0, same as the claudy project (see repository root `LICENSE`). The skill is advisory guidance; Hermes always owns the Kanban lifecycle and final acceptance.
