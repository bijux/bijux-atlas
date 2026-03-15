# Repository Config Sources

This directory groups authored inputs that govern repository automation and maintainer workflows.

Current domains:
- `ci/` for lane metadata and environment contracts.
- `coverage/` for repository coverage thresholds.
- `docs/` for documentation linting and build inputs.
- `gates/` for repository gate routing metadata.
- `layout/` for tree-budget and repository-shape rules.
- `make/` for published make surface inputs.
- `meta/` for repository ownership metadata that is still config-like rather than registry-owned.
- `nextest/` for test-runner configuration.
- `repo-surface/` for repository surface snapshots and allowlists.
- `rust-tooling/` for rustfmt, clippy, and toolchain inputs.
- `shell/` for shell lint configuration.

These files are maintainer-facing sources of truth. Product runtime inputs belong under `configs/sources/runtime/`.
