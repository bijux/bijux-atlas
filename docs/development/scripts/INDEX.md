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
  - `scripts/areas/internal/`: script-only helpers.
  - Contracts: `atlasctl contracts ...`
- Docs linters/generators: `atlasctl docs ...`
- Perf tooling wrappers: `scripts/areas/public/perf/` (canonical: `ops/load/scripts/`)
- Observability checks: `scripts/areas/public/observability/`
- Fixtures/data helpers: `scripts/areas/fixtures/`
- Release automation: `scripts/areas/release/`
- Layout checks/migrations: `scripts/areas/layout/`
- Bootstrap wrappers and runtime helpers: `scripts/bin/`

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
