# Ops

- Owner: `bijux-atlas-operations`

## What

Canonical operational filesystem surface and only entrypoint for ops workflows.

## Directory map

- `ops/stack/`: local stack manifests and bootstrap scripts (kind/minio/prom/otel/redis/toxiproxy).
- `ops/e2e/`: e2e harness, test runners, smoke/publish/deploy workflows.
- `ops/load/`: k6 suites, scenarios, contracts, baselines, perf scripts.
- `ops/observability/`: alerts, dashboards, observability contracts and drills.
- `ops/k8s/`: chart, values profiles, k8s CI scripts.
- `ops/_lib/`: shared shell helpers for ops scripts/tests.
- `ops/tool-versions.json`: pinned ops tool versions consumed by `make ops-tools-check`.
- `ops/fixtures/`, `ops/datasets/`: pinned ops datasets and fixture metadata.
- `ops/e2e/`: harness + runners only (tests and orchestration).
- `ops/smoke/`: locked smoke query set + goldens + smoke report generator.
- `ops/ui/`: helper scripts that print local service URLs.

## Run full stack

```bash
make ops-full
```

All sub-area docs under `ops/**/README.md` should point here as the primary entrypoint.

## Why

Single ops SSOT avoids root-path alias drift.

## See also

- `docs/operations/INDEX.md`
- `docs/operations/full-stack-local.md`
