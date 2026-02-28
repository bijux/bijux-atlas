# Repository Layout

- Owner: `docs-governance`

## What

Defines the allowed root-level repository entries.

## Why

Prevents root sprawl and keeps `ops/` as the single operational source of truth.

## Contracts

- Root allowlist is defined in `crates/bijux-dev-atlas/src/checks/layout/root_whitelist.json`.
- Legacy root aliases are forbidden: `charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`.
- Root shape gate: `bijux dev atlas check root-shape`.
- Forbidden-name gate: `bijux dev atlas check forbidden-root-names`.
- Migration entrypoint: `make layout-migrate`.
- Python package surfaces live under `crates/` only.
- New executable Python files outside package roots are forbidden by `bijux dev atlas` repository policy checks.
- Removed script-tree paths must not be reintroduced.

## Failure modes

Unexpected root entries or reintroduced legacy aliases fail CI and local layout checks.

## How to verify

```bash
$ make check-gates
$ make layout-migrate
```

Expected output: both commands complete without layout errors.

## See also

- [Repo Surface](repo-surface.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
- [Symlink Index](symlinks.md)
