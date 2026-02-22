# Repository Layout

- Owner: `docs-governance`

## What

Defines the allowed root-level repository entries.

## Why

Prevents root sprawl and keeps `ops/` as the single operational source of truth.

## Contracts

- Root allowlist is defined in `packages/atlasctl/src/atlasctl/checks/layout/root_whitelist.json`.
- Legacy root aliases are forbidden: `charts`, `e2e`, `load`, `observability`, `datasets`, `fixtures`.
- Root shape gate: `atlasctl check root-shape`.
- Forbidden-name gate: `atlasctl check forbidden-root-names`.
- Migration entrypoint: `make layout-migrate`.
- Python package surfaces live under `packages/` and `tools/`.
- New executable Python files outside package roots are forbidden by `atlasctl` repository policy checks.
- Legacy script-tree paths are transition-only and being removed; no new non-shim entrypoints may be added.

## Failure modes

Unexpected root entries or reintroduced legacy aliases fail CI and local layout checks.

## How to verify

```bash
$ make atlasctl-check-layout
$ make layout-migrate
```

Expected output: both commands complete without layout errors.

## See also

- [Repo Surface](repo-surface.md)
- [Ops Canonical Layout](ops-canonical-layout.md)
- [Symlink Index](symlinks.md)
