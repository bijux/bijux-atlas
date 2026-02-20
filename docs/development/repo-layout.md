# Repository Layout

- Owner: `docs-governance`

## What

Defines the allowed root-level repository entries.

## Why

Prevents root sprawl and keeps `ops/` as the single operational source of truth.

## Contracts

- Root allowlist is defined in `scripts/areas/layout/root_whitelist.json`.
- Legacy root aliases are forbidden: `charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`.
- Root shape gate: `scripts/areas/layout/check_root_shape.sh`.
- Forbidden-name gate: `scripts/areas/layout/check_forbidden_root_names.sh`.
- Migration entrypoint: `make layout-migrate`.

## Failure modes

Unexpected root entries or reintroduced legacy aliases fail CI and local layout checks.

## How to verify

```bash
$ make layout-check
$ make layout-migrate
```

Expected output: both commands complete without layout errors.

## See also

- [Repo Surface](repo-surface.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
- [Symlink Index](symlinks.md)
