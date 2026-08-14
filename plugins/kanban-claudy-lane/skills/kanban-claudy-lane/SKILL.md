---
name: kanban-claudy-lane
description: Use when a Hermes Kanban worker wants to run claudy (the Claude CLI launcher) as an isolated implementation lane while Hermes keeps ownership of task lifecycle, reconciliation, testing, and handoff. Codex-substitute for users without a Codex subscription.
version: 1.1.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [kanban, claudy, claude-cli, worktrees, autonomous-agents]
    related_skills: [kanban-worker, kanban-codex-lane, hermes-agent]
---

# Kanban Claudy Lane

## Overview

This skill defines the lightweight Hermes+claudy dual-lane convention for Kanban workers. Hermes is always the task owner: it calls `kanban_show`, decides whether delegation is appropriate, creates or selects an isolated workspace, runs claudy in it, reconciles any diff, runs verification, and writes the final `kanban_complete` or `kanban_block` handoff. claudy is an input lane only. Its output is not a task completion signal, not a trusted reviewer, and not allowed to write durable Kanban state directly.

The convention exists so a Hermes worker can use claudy for bounded implementation help without changing the dispatcher. The dispatcher must still spawn Hermes workers. A worker may optionally delegate implementation inside its own run, then accept, partially accept, or reject the lane after independent review and tests.

claudy is the user's Claude CLI launcher (`claudy -p "PROMPT" --yolo`, where `--yolo` maps to `--dangerously-skip-permissions`). It runs the Claude CLI in the current working directory. This is the Codex-substitute lane: same ownership model as `kanban-codex-lane`, but it shells out to claudy instead of `codex`, for environments without a Codex subscription.

## When to Use

Use the claudy lane when all of these are true:

- The Kanban task is a coding, refactor, documentation, test, or mechanical migration task with clear acceptance criteria.
- A bounded diff can be evaluated by Hermes in one run.
- The repo can be copied or checked out in an isolated git worktree/branch.
- Hermes can run the relevant tests itself after claudy exits.
- The prompt can state all safety constraints and files that must not change.
- claudy is installed (`command -v claudy`).

Do not use the claudy lane when any of these are true:

- The task requires human judgment that is not already captured in the Kanban body.
- The worker lacks repo access or time to reconcile the result.
- The change touches secrets, credential stores, private user data, or production order-entry systems.
- A small direct edit is faster and safer than spawning another agent.
- The task is research-only and should produce a written handoff rather than a diff.
- The worker would be tempted to mark Done based only on claudy's self-report.

## Ownership Rules

1. Hermes owns the Kanban lifecycle. claudy must never call `kanban_complete`, `kanban_block`, `kanban_create`, gateway messaging, or any Hermes board CLI as a substitute for the worker.
2. Hermes owns final acceptance. Treat claudy's diff/commits as untrusted patches until reviewed and verified.
3. Hermes owns test execution. claudy may run tests, but those runs are advisory; repeat required verification from Hermes with the repo's canonical wrapper.
4. Hermes owns safety. If claudy changes safety boundaries, risk gates, live behavior, or secrets handling, reject the lane even if tests pass.
5. Hermes owns cleanup. Kill stuck claudy processes and remove temporary worktrees when they are no longer needed.

## Required Worktree and Branch Pattern

Never run claudy directly in a shared dirty checkout. claudy runs in its cwd, so isolation = point it at a dedicated worktree. Use a branch/worktree name that ties the lane to the Kanban task.

Recommended variables:

```bash
TASK_ID="${HERMES_KANBAN_TASK:-t_manual}"
REPO="/path/to/repo"
BASE="$(git -C "$REPO" rev-parse --abbrev-ref HEAD)"
SAFE_TASK="$(printf '%s' "$TASK_ID" | tr -cd '[:alnum:]_-')"
BRANCH="claudy/${SAFE_TASK}/$(date -u +%Y%m%d%H%M%S)"
WORKTREE="/tmp/${SAFE_TASK}-claudy-lane"
```

Create the isolated lane:

```bash
git -C "$REPO" fetch --all --prune
git -C "$REPO" worktree add -b "$BRANCH" "$WORKTREE" "$BASE"
git -C "$WORKTREE" status --short --branch
```

If the current Kanban workspace is already an isolated git worktree created for this task, you may create a sibling branch inside it only if `git status --short` is clean except for intentional Hermes edits. Otherwise create a separate temporary worktree and cherry-pick or copy accepted commits back after reconciliation.

Cleanup after reconciliation:

```bash
git -C "$REPO" worktree remove "$WORKTREE"
git -C "$REPO" branch -D "$BRANCH"  # only after accepted commits were copied/cherry-picked or intentionally rejected
```

Keep the worktree if it is needed as an artifact for review; record it in `claudy_lane.artifacts` and mention it in the handoff.

## Claudy Capability Check

Run this before delegating. A missing claudy is a normal reason to skip the lane, not a task blocker if Hermes can do the task directly.

```bash
command -v claudy && claudy --version
```

If claudy is not installed, skip the lane (`claudy_lane.used: false`, `rejected_reason: "claudy not installed"`) and do the work directly or block.

## Invocation

claudy takes the prompt with `-p` and runs autonomously with `--yolo` (maps to `--dangerously-skip-permissions`). It executes in its cwd; the worktree-as-cwd is all the scoping it needs — claudy takes no path argument.

An optional first positional selects the provider profile (`claudy list` shows them, e.g. `native`, `zai-coding`). When the current build requires it, prefix the profile; otherwise the bare form works:

```bash
claudy -p "$(cat /tmp/claudy-lane-prompt.md)" --yolo
# or, profile-pinned:
claudy zai-coding -p "$(cat /tmp/claudy-lane-prompt.md)" --yolo
```

Launch via the `terminal` tool with the worktree as `workdir`, a PTY, and completion notification so Hermes can monitor without blocking:

```python
terminal(
    command='claudy -p "$(cat /tmp/claudy-lane-prompt.md)" --yolo',
    workdir=WORKTREE,
    background=True,
    pty=True,
    notify_on_complete=True,
)
```

`-p` is print/non-interactive mode — claudy produces its response and exits, so a single run is bounded.

## Prompt Construction

Use the linked template at `templates/claudy-lane-prompt.md`. Fill every `[BRACKETED]` field before launch. For repo-specific safety, replace the `[SAFETY_CONSTRAINTS]` block with the repo's invariants.

Every claudy-lane prompt must include:

- `task_id`, title, and full Kanban acceptance criteria.
- Repo path, worktree path, branch name, and allowed file scope.
- Explicit statement: Hermes owns Kanban lifecycle; claudy is an input lane only.
- Required output: concise summary, files changed, commits, tests run, and known risks.
- Prohibited actions: secrets access, external messaging, board mutation, unrelated refactors, dependency upgrades unless required.
- Verification commands claudy may run and commands Hermes will run afterward.
- The `[SAFETY_CONSTRAINTS]` block, filled with repo-specific invariants (see template).

## Monitoring, Timeout, and Kill Behavior

Start the lane in the background with PTY and completion notification (see Invocation). Monitor without interfering:

```python
process(action="poll", session_id=session_id)
process(action="log", session_id=session_id, limit=200)
process(action="wait", session_id=session_id, timeout=600)
```

Send a Kanban heartbeat every few minutes for lanes longer than two minutes, e.g. `kanban_heartbeat(note="claudy lane running in $WORKTREE; waiting for diff")`.

Kill conditions:

- No useful output for the task's remaining runtime budget.
- claudy requests secrets, production credentials, or external permissions.
- claudy attempts to modify files outside the worktree.
- claudy starts unrelated rewrites or dependency churn.
- claudy is still running near the worker timeout and no safe partial artifact exists.

Kill command:

```python
process(action="kill", session_id=session_id)
```

After kill, inspect `git status --short`, preserve useful patches only if safe, and record `claudy_lane.result: timed_out` or `rejected` with a concrete `rejected_reason`.

## Reconciliation Checklist

Hermes must perform this checklist before accepting any claudy-lane result:

- [ ] `git -C <WORKTREE> status --short --branch` shows only expected files.
- [ ] `git -C <WORKTREE> diff --stat` and `git diff` were reviewed by Hermes.
- [ ] No secrets, credentials, generated caches, unrelated data, or local artifacts are included.
- [ ] Repo-specific `[SAFETY_CONSTRAINTS]` were preserved.
- [ ] claudy commits are small enough to cherry-pick or squash cleanly.
- [ ] Hermes ran the canonical tests itself, using `scripts/run_tests.sh` for Hermes Agent or the repo's documented wrapper for other repos.
- [ ] Any claudy-run tests are listed separately from Hermes-run tests.
- [ ] Accepted commits/diffs were applied to the Hermes-owned workspace/branch.
- [ ] Rejected or partial work has a concrete reason and artifact path if useful.

Acceptance outcomes:

- `accepted`: claudy diff/commits were reviewed, applied, and verified.
- `partial`: some claudy work was accepted after edits or cherry-picks; rejected parts are documented.
- `rejected`: no claudy changes were accepted; reason is documented.
- `timed_out`: claudy exceeded the lane budget; useful artifacts may or may not exist.

## kanban_complete Metadata Schema

Include this object under `metadata.claudy_lane` for every task where the lane was considered. If claudy was not used, set `used: false` and explain why in `rejected_reason` or a sibling `notes` field.

```json
{
  "claudy_lane": {
    "used": true,
    "worktree": "/absolute/path/to/worktree",
    "branch": "claudy/t_xxx/20260508100000",
    "command": "claudy -p ... --yolo",
    "result": "accepted | rejected | partial | timed_out",
    "accepted_commits": ["<sha1>", "<sha2>"],
    "rejected_reason": "empty when fully accepted; otherwise concrete reason",
    "tests_run": [
      {"command": "scripts/run_tests.sh tests/tools/test_x.py", "exit_code": 0, "owner": "hermes"},
      {"command": "claudy-reported: npm test", "exit_code": 0, "owner": "claudy"}
    ],
    "artifacts": ["/absolute/path/to/log-or-patch"]
  }
}
```

For tasks that intentionally skip the lane:

```json
{
  "claudy_lane": {
    "used": false,
    "worktree": null,
    "branch": null,
    "command": null,
    "result": "rejected",
    "accepted_commits": [],
    "rejected_reason": "Direct Hermes edit was smaller and safer than delegating.",
    "tests_run": [],
    "artifacts": []
  }
}
```

## Common Pitfalls

1. Treating claudy's self-report as verification. Always inspect the diff and rerun tests from Hermes.
2. Running claudy in the user's dirty main checkout. Always isolate in a worktree/branch and launch claudy with that worktree as cwd.
3. Letting claudy own Kanban. It may summarize progress, but Hermes writes board state.
4. Forgetting repo-specific `[SAFETY_CONSTRAINTS]` in the prompt. Missing safety text is a lane setup failure.
5. Killing a stuck lane without recording why. `rejected_reason` must explain the decision.
6. Accepting broad unrelated cleanup because tests pass. Reject or cherry-pick only the scoped changes.
7. Passing a repo path argument. claudy takes `-p "<prompt>" --yolo` and runs in cwd — scope it via the worktree, nothing else.

## Verification Checklist

- [ ] claudy was skipped or started only after the capability check confirmed it is installed + version-printed.
- [ ] claudy ran only in an isolated worktree/branch (workdir pinned to the worktree).
- [ ] Prompt included task scope, ownership rules, `[SAFETY_CONSTRAINTS]`, and verification commands.
- [ ] Hermes reviewed `git diff` and safety-sensitive files.
- [ ] Hermes ran canonical tests independently.
- [ ] `kanban_complete.metadata.claudy_lane` follows the schema above.
- [ ] Temporary processes and unnecessary worktrees were cleaned up.
