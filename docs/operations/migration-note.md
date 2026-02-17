# Ops Path Migration Note

- Owner: `bijux-atlas-operations`

## What

Maps legacy root aliases to canonical `ops/` paths.

## Mapping

- `./charts/` -> `./ops/k8s/charts/`
- `./e2e/` -> `./ops/e2e/`
- `./load/` -> `./ops/load/`
- `./observability/` -> `./ops/observability/`
- `./datasets/` -> `./ops/datasets/`
- `./fixtures/` -> `./ops/fixtures/`

## Commands

Use make targets only:

```bash
$ make layout-migrate
$ make layout-check
```

Operational validation path: run `ops-full` after migration.

## See also

- [No Direct Path Usage](no-direct-path-usage.md)
- [Ops Layout](ops-layout.md)
