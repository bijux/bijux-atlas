# Scripts Index

- Owner: `docs-governance`

## What

Defines script interfaces grouped by domain.

## Why

Scripts are operational APIs and require stable discoverability.

## Scope

Scripts under `scripts/` and e2e-facing script surfaces.

## Non-goals

Does not duplicate script implementation details.

## Contracts

- `scripts` inventory is generated via `atlasctl inventory scripts-migration`.
- Script taxonomy:
  - `scripts/areas/public/`: make-callable entrypoints.
  - Contracts: `atlasctl contracts ...`
- Docs linters/generators: `atlasctl docs ...`
- Perf tooling wrappers: `scripts/areas/public/perf/` (canonical: `ops/load/scripts/`)
- Observability checks: `packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/`
- Fixtures/data helpers: `ops/datasets/scripts/fixtures/`
- Release compatibility matrix automation: `atlasctl compat update-matrix|validate-matrix`
- Layout checks/migrations: `scripts/areas/layout/`
- Runtime helpers: `atlasctl env ...` and ops-native script entrypoints under `ops/`

## Failure modes

Undocumented scripts cause hidden coupling and broken operator workflows.

## How to verify

```bash
$ atlasctl docs script-headers-check --report text
```

Expected output: script header contract passes.

## See also

- [Repo Surface](../repo-surface.md)
- [Makefile Surface](../makefiles/surface.md)
- [E2E Scripts](../../operations/e2e/scripts.md)
