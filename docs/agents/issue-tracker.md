# Issue Tracker

Issues are tracked in GitHub at **epicsagas/claudy** using `gh` CLI.

## Prerequisites

Verify `gh` is authenticated:

```bash
gh auth status
```

If not authenticated, run `gh auth login` and follow the prompts.

## Issue Templates

Two templates are available in `.github/ISSUE_TEMPLATE/`:

- **Bug Report** (`bug_report.yml`) — Structured form for reproducible defects
- **Feature Request** (`feature_request.yml`) — Structured form for enhancements

## Auto-labeling

Issues created via templates are automatically labeled:

| Template | Auto-applied labels |
|---|---|
| Bug Report | `bug`, `needs-triage` |
| Feature Request | `enhancement`, `needs-triage` |
| Dependabot | `dependencies`, `needs-triage` |

## Creating Issues

```bash
# Bug report (interactive)
gh issue create --template bug_report.yml

# Feature request (interactive)
gh issue create --template feature_request.yml

# Quick issue with labels
gh issue create --title "fix: description" --label "bug,needs-triage"
```

## Linking Issues to PRs

Use Conventional Commits with issue references:

```
fix(channel): resolve webhook timeout on large payloads

Closes #42
```

Keywords: `Closes #N`, `Fixes #N`, `Resolves #N` — these auto-close the issue on merge.

## Security Issues

**Do NOT file security vulnerabilities as GitHub issues.** Follow the process described in `SECURITY.md` instead.

## Common Commands

```bash
# List open issues
gh issue list --state open

# View issue details
gh issue view 42

# Add a label
gh issue edit 42 --add-label "ready-for-agent"

# Close an issue
gh issue close 42 --reason "completed"
```
