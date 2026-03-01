# Contributing

## Scope
Bijux Atlas ships two command surfaces:
- `bijux atlas` for runtime product actions
- `bijux dev atlas` for repo checks, contracts, docs, configs, and ops governance

## Repo Laws
- RL-001: Executable script sources are forbidden unless allowlisted fixtures.
- RL-002: Make is a thin dispatcher; orchestration belongs in Rust control-plane code.
- RL-003: `artifacts/` is runtime output and never tracked.
- RL-004: Root directories and root markdown files are explicit allowlists.
- RL-005: Duplicate SSOT registries are forbidden.
- RL-006: Generated outputs must be deterministic.
- RL-007: Repo-law records require `id`, `severity`, and `owner`.
- RL-008: PR required suite must include all defined P0 checks.
- RL-009: Defaults for build/docs/helm-template flows must work.
- RL-010: New root files require explicit allowlist approval.

Canonical source: `docs/_internal/contracts/repo-laws.json`.

## Local Validation
- `make fmt`
- `make lint`
- `make test`
- `make check`

## Ownership
See `.github/CODEOWNERS` and `docs/reference/repo-map.md`.
