# Ops Layout

- Owner: `bijux-atlas-operations`

## What

Canonical operational filesystem layout.

## Directory map

- `ops/stack/`: stack manifests and bootstrap scripts.
- `ops/e2e/`: harness and test runners only.
- `ops/k8s/`: charts, values, and tests.
- `ops/load/`: k6 suites and scoring.
- `ops/observability/`: dashboards, alerts, and contracts.
- `ops/datasets/`, `ops/fixtures/`: dataset and fixture assets.
- `ops/_lib/`: shared shell helpers.

## Run full stack

```bash
$ make ops-full
```

Canonical meta target: `ops-full`.

## See also

- [Operations Index](INDEX.md)
- [Full Stack Local](full-stack-local.md)
