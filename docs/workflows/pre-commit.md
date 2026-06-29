# Pre-Commit Workflow

Run these checks before committing changes:

```bash
cargo fmt --all -- --check
cargo test
cargo check
git diff --check
```

## Local Git Hook

This project stores versioned hooks in `.githooks`.

Enable them once per clone:

```bash
git config core.hooksPath .githooks
```

After enabling, `git commit` runs `.githooks/pre-commit` automatically.

## Scope

The hook is intentionally lightweight for early development:

- Checks Rust formatting.
- Runs the test suite.
- Verifies the project compiles.
- Detects whitespace errors in staged and unstaged diffs.

Coverage, release automation, security audit, and extra test runners can be added later when the project stabilizes.
