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

- Contracts: `scripts/contracts/`
- Docs linters/generators: `scripts/docs/`
- Perf tooling: `scripts/perf/`
- Observability checks: `scripts/observability/`
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
- [E2E Scripts](../../operations/ops/e2e/scripts.md)
