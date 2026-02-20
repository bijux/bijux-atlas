# Migration Map: Ops Runtime

Scope: non-lint `ops/*` scripts (datasets, load, obs, report, run, stack, e2e, k8s tests).

## Table Contract

| Legacy Script | New Module Path | New CLI Command | Output Schema | Tests |
|---|---|---|---|---|

## Notes

- Group by ops domain (`datasets`, `load`, `obs`, `report`, etc.) in stable sections.
- Keep deterministic output contract notes when commands are report-producing.
