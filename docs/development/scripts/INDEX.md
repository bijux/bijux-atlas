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

- `scripts/INDEX.md` is generated (`python3 scripts/generate_scripts_readme.py`) and must not be hand-edited.
- Script taxonomy:
  - `scripts/public/`: make-callable entrypoints.
  - `scripts/internal/`: script-only helpers.
  - `scripts/dev/`: local helpers (not docs/CI contracts).
  - `scripts/tools/`: shared Python helper modules.
- Contracts: `scripts/contracts/`
- Docs linters/generators: `scripts/docs/`
- Perf tooling wrappers: `scripts/public/perf/` (canonical: `ops/load/scripts/`)
- Observability checks: `scripts/public/observability/`
- Fixtures/data helpers: `scripts/fixtures/`
- Release automation: `scripts/release/`
- Layout checks/migrations: `scripts/layout/`
- Bootstrap wrappers and runtime helpers: `scripts/bin/`

## Failure modes

Undocumented scripts cause hidden coupling and broken operator workflows.

## How to verify

```bash
$ python3 scripts/docs/check_script_headers.py
```

Expected output: script header contract passes.

## See also

- [Repo Surface](../repo-surface.md)
- [Makefile Surface](../makefiles/surface.md)
- [E2E Scripts](../../operations/e2e/scripts.md)
