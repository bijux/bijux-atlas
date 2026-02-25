# Dashboard Contract

- Owner: `bijux-atlas-operations`

## What

Describes required dashboard panels and their metric dependencies.

## Why

Keeps Grafana panels aligned with metric contracts.

## Scope

`ops/observe/grafana/atlas-observability-dashboard.json` and required metrics.

## Non-goals

Does not define collector deployment topology.

## Contracts

- Dashboard panels must reference metrics declared in [`docs/contracts/metrics.md`](../../contracts/metrics.md).
- Panel labels must avoid user-controlled cardinality.
- Dashboard JSON includes explicit `schemaVersion` and `version` fields.

## Failure modes

Dashboard drift hides production regressions.

## How to verify

```bash
$ make ops-dashboards-validate
$ make observability-check
```

Expected output: dashboard and metrics contracts pass.

## See also

- [Observability Index](INDEX.md)
- [Alerts](alerts.md)
- [Metrics Contract](../../contracts/metrics.md)
- `ops-ci`
