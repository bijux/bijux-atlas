# Control-Plane Migration Rationale

The repository uses a Rust control plane (`bijux-dev-atlas`) so governance and developer workflows are:

- versioned with the workspace crates
- testable with Rust tooling
- capability-gated (`--allow-subprocess`, `--allow-write`, `--allow-network`)
- deterministic in CI output and artifacts

Makefiles and workflows are wrapper layers only. They delegate to `bijux dev atlas ...` and should not contain business logic.
