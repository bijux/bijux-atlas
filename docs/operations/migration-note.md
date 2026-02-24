# Ops Path Migration Note

- Owner: `bijux-atlas-operations`

## What

Maps legacy root aliases to canonical `ops/` paths.

## Mapping

- `./charts/` -> `./ops/k8s/charts/`
- `./e2e/` -> `./ops/e2e/`
- `./load/` -> `./ops/load/`
- `./observability/` -> `./ops/obs/`
- `./datasets/` -> `./ops/datasets/`
- `./fixtures/` -> `./ops/fixtures/`

## Wording Migration

- `Reference Grade Checklist` -> `Release Contract Checklist`
- Legacy marketing wording -> `contract` / `policy` wording in docs and command output
- Blocked-term enforcement uses explicit policy config and explicit approvals for historical quotes only

## Commands

Use make targets only:

```bash
$ make layout-migrate
$ make check-gates
```

Operational validation path: run `ops-full` after migration.

## See also

- [No Direct Path Usage](no-direct-path-usage.md)
- [Ops Layout](ops-layout.md)
