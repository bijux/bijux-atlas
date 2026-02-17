# Ops Layout

- Owner: `bijux-atlas-operations`

## What

Canonical operational filesystem surface for atlas.

## Directory map

- `ops/e2e/`: cluster stack scripts, k8s test harness, realdata drills.
- `ops/load/`: k6 suites, scenarios, contracts, baselines, perf scripts.
- `ops/observability/`: alerts, dashboards, observability contracts and drills.
- `ops/openapi/`: generated OpenAPI artifacts and snapshots.
- `ops/k8s/`: chart, values profiles, k8s CI scripts.
- `ops/stack/`: local stack manifests (minio/prom/otel/redis/toxiproxy).
- `ops/fixtures/`, `ops/datasets/`: pinned ops datasets and fixture metadata.

## Run full stack

```bash
make ops-up
make ops-deploy
make ops-warm
make ops-smoke
```

## Why

Single ops SSOT avoids root-path alias drift.

## See also

- `docs/operations/INDEX.md`
- `docs/operations/full-stack-locally.md`
