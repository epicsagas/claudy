# Contributing to Claudy

Claudy is a modern multi-provider launcher for Claude CLI, and we welcome contributions of all kinds.

## Prerequisites

- Rust 1.92 or later
- Claude CLI installed and accessible in your PATH

## Getting Started

1. Fork the repository on GitHub.

2. Clone your fork locally:

   ```
   git clone https://github.com/<your-username>/claudy.git
   cd claudy
   ```

3. Create a feature branch. Use one of the following prefixes:

   - `feat/` — new feature
   - `fix/` — bug fix
   - `chore/` — tooling, dependency, or non-functional change

   ```
   git checkout -b feat/my-feature
   ```

4. Make your changes and commit following the [Commit Message](#commit-message) conventions below.

5. Push your branch and open a Pull Request against `main`.

## Code Style

Format all code before committing:

```
cargo fmt
```

Lint with zero warnings allowed:

```
cargo clippy -- -D warnings
```

## Testing

Run the full test suite:

```
cargo test
```

All existing tests must pass. New functionality must be accompanied by appropriate tests.

## Pull Request Checklist

Before submitting a PR, confirm the following:

- [ ] `cargo fmt` has been run and the diff is clean
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo test` passes locally
- [ ] New code has test coverage where applicable
- [ ] The PR description explains what was changed and why
- [ ] Related issues are referenced (e.g., `Closes #42`)

## Reporting Issues

### Bug Reports

Open a GitHub Issue and include:

- A clear, descriptive title
- Steps to reproduce the problem
- Expected behavior vs. actual behavior
- Claudy version (`claudy --version`) and OS/platform
- Any relevant logs or error output

### Feature Requests

Open a GitHub Issue with the `enhancement` label and include:

- A concise description of the proposed feature
- The problem it solves or the use case it enables
- Any alternative approaches you have considered

## Commit Message

This project follows [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

Format:

```
<type>(<optional scope>): <short summary>
```

Allowed types:

| Type | When to use |
|------|-------------|
| `feat` | A new feature |
| `fix` | A bug fix |
| `docs` | Documentation changes only |
| `chore` | Build process, dependency updates, or tooling changes |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `test` | Adding or correcting tests |
| `perf` | Performance improvement |

Examples:

```
feat(profile): add multi-account switching
fix(cli): handle missing config file gracefully
docs: update CONTRIBUTING with clippy instructions
chore: bump Rust edition to 2024
```

## Code of Conduct

Be respectful and constructive. Harassment, discrimination, and toxic behavior are not tolerated.
