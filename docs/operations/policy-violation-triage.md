# Policy Violation Triage

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `atlas-platform`
- Stability: `evolving`

## Why

Policy rejections must be diagnosable from metrics and logs without code spelunking.

## What To Check

1. `atlas_policy_violations_total{policy=...}` to identify the active policy gate.
2. `atlas_policy_relaxation_active{mode=...}` to confirm runtime policy mode.
3. `bijux_overload_shedding_active` and `atlas_overload_active` for load-shed context.
4. Request logs with `request_id`, `query_class`, and policy budget fields.

## How

1. Run `make ops-observability-validate`.
2. Inspect `artifacts/ops/<run-id>/observability/metrics.prom`.
3. Correlate rejected request IDs in logs with policy metric spikes.

## Failure Modes

- If `atlas_policy_relaxation_active` is `1` unexpectedly, verify `ATLAS_POLICY_MODE`.
- If violations spike under normal load, compare query budgets in `configs/policy/policy.json`.

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
