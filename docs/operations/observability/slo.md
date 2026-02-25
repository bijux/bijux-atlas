# Bijux Atlas SLO

Canonical targets are defined in `docs/product/slo-targets.md`.
This file describes service-level interpretation and error-budget operations.

## What

Defines observability SLO interpretation and burn policy for operations.

## Why

Provides a stable decision contract for reliability gates and incident response.

## Contracts

SLO targets and burn math are contractually evaluated from runtime metrics.

## Service Objectives
- `v1/genes` p95 latency: <= 800 ms (steady read load)
- `v1/genes/count` p95 latency: <= 300 ms
- 5xx error rate: < 0.5% over rolling 30 days

## Error Budget
- Monthly error budget: 0.5% of total requests
- Budget burn policy:
  - fast burn (>10% budget/day): freeze risky deploys
  - sustained burn (>2% budget/day for 3 days): prioritize reliability backlog
- Runtime burn formula (from metrics snapshot):
  - `error_rate = 5xx_requests / total_requests`
  - `burn_rate = error_rate / 0.005`
  - `burn_exceeded = burn_rate > 1.0`

## Valid Degradation Modes
- Cached-only serving when store backend is degraded.
- Provisional request shedding via rate/concurrency limits.
- Strict response-size rejection instead of partial/truncated payloads.

## Related Contracts
- Alert rules: `ops/observe/alerts/atlas-alert-rules.yaml`
- Metrics contract: `ops/observe/contracts/metrics-contract.json`
- Metric cardinality guardrails: `docs/operations/observability/metric-cardinality-guardrails.md`

## How to verify

```bash
make ops-observability-validate
make ops-slo-burn
cat artifacts/ops/observe/slo-burn.json
```

## Failure modes

Missing or inconsistent metrics produce invalid burn decisions and incorrect gating.

## See also

- `ops-ci`
