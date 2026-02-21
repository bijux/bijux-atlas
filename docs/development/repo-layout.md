# Repository Layout

- Owner: `docs-governance`

## What

Defines the allowed root-level repository entries.

## Why

Prevents root sprawl and keeps `ops/` as the single operational source of truth.

## Contracts

- Root allowlist is defined in `packages/atlasctl/src/atlasctl/checks/layout/root_whitelist.json`.
- Legacy root aliases are forbidden: `charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`.
- Root shape gate: `packages/atlasctl/src/atlasctl/checks/layout/shell/check_root_shape.sh`.
- Forbidden-name gate: `packages/atlasctl/src/atlasctl/checks/layout/shell/check_forbidden_root_names.sh`.
- Migration entrypoint: `make layout-migrate`.
- Python package surfaces live under `packages/` and `tools/`.
- New executable Python files outside package roots are forbidden by `scripts/areas/check/check-no-python-executable-outside-tools.py`.
- Legacy `scripts/` is transition-only and being removed; no new non-shim entrypoints may be added.

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
