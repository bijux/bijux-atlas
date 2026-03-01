# Repo Laws

Canonical source: this directory is the single governance source for repository-level laws.

## Laws

- `RL-001`: Executable script sources (`*.sh`, `*.py`, `*.js`, `*.rb`, `*.pl`) are forbidden unless explicitly allowlisted as fixtures.
- `RL-002`: `make` is a thin dispatcher to `bijux-dev-atlas`; orchestration logic belongs in Rust control-plane commands.
- `RL-003`: `artifacts/` is runtime output and must never be tracked by git.
- `RL-004`: Top-level repository paths are allowlisted and controlled.
- `RL-005`: Duplicate SSOT registries (metadata/owners/registry/sections) are forbidden.
- `RL-006`: Generated outputs must be deterministic (stable ordering, stable formatting).
- `RL-007`: Repo-law records require `id`, `severity`, and `owner`.
- `RL-008`: Pull request required suites must execute all defined P0 checks.
- `RL-009`: Default developer flows must work for build, docs, and helm template surfaces.
- `RL-010`: New root files require explicit root allowlist approval.
