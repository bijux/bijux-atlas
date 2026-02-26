# Canary Progressive Delivery Notes

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

`bijux-atlas` can run either a standard `Deployment` or an Argo `Rollout`.

## Recommended Sequence

1. Deploy with `rollout.enabled=true` and low initial canary weight.
2. Observe p95 latency, 5xx rate, and cache churn for at least one window.
3. Advance canary to 50% only if error budget burn is acceptable.
4. Promote to 100% only after sustained steady-state metrics.

## Abort Conditions

Abort rollout when any of these persist for more than 5 minutes:

- Elevated `5xx` above baseline SLO threshold.
- p95 latency materially above target.
- Dataset cache thrash or repeated store/open failures.

## Operational Requirements

- Argo Rollouts controller installed.
- Prometheus scraping `/metrics`.
- Alert rules active for 5xx and latency.
## Referenced chart values keys

- `values.rollout`
- `values.hpa`

## See also

- `ops-ci`
