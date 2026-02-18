# Warm-On-Rollout Hook

- Owner: `bijux-atlas-operations`
- Stability: `evolving`

## Goal

Warm pinned datasets during rollout so first user requests do not pay cold-cache latency.

## Contract

- Hook uses pinned dataset list from `cache.pinnedDatasets` / `ATLAS_PINNED_DATASETS`.
- Warm action must run before service is considered ready for traffic.
- Warm failures may either block readiness (strict mode) or log and continue (lenient mode).

## Deployment Patterns

1. Init container prewarm:
   - Runs before app container starts.
   - Best for strict startup correctness.
2. Post-deploy Job warmup:
   - Runs after deployment and can be retried independently.
   - Better when warm data set is large.

## Local Ops Entrypoints

- `make ops-cache-pin-set DATASETS=release/species/assembly[,..]`
- `make ops-deploy PROFILE=local`
- `make ops-warm`
- `make ops-warm-datasets DATASETS=release/species/assembly[,..]`
- `make ops-warm-top TOP_N=5`

## Failure Modes

- Store outage with empty cache: requests for uncached datasets fail in cached-only mode.
- Misconfigured pin list: warm hook runs but does not touch expected datasets.
