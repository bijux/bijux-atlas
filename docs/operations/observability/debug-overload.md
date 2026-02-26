# Debug Overload

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## Signals

- Metrics: `atlas_overload_active`, `atlas_shed_total`, `atlas_bulkhead_inflight`, `atlas_bulkhead_saturation`.
- Logs: policy rejection events with policy + mode + reason + limit.
- Runbooks: `docs/operations/runbooks/traffic-spike.md`.

## Commands

- `bijux dev atlas ops observe drill run overload-admission-control`
- `make observability-pack-drills`
- `make ops-report`
