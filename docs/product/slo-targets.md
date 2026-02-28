# SLO Targets

- Owner: `bijux-atlas-operations`

## What

Service-level objectives sourced from `configs/slo/slo.json`.

## Why

SLOs must be machine-checked and scenario-specific.

## Scope

Applies to performance gates and nightly suites.

## Non-goals

No manual threshold duplication in docs.

## Contracts

- `configs/slo/slo.json` is SSOT.
- Global and per-scenario budgets are enforced by perf scoring scripts.
- Required metrics in SLO config must exist in `/metrics`.

## Interpretation

- `p95_ms_max` and `p99_ms_max` are latency ceilings per scenario.
- `error_rate_max` is maximum failed-request fraction.
- `cold_start_p99_ms_max` covers first-request latency after startup.

## Failure modes

Scenario score above thresholds fails nightly gates.

## How to verify

```bash
$ rg -n "p95_ms_max|p99_ms_max|error_rate_max|cold_start_p99_ms_max" configs/slo/slo.json
$ make ops-nightly
```

Expected output: SLO file is parsed and perf scoring passes.

## See also

- [Observability SLO Policy](../operations/observability/slo-policy.md)
- [k6 Load Scenarios](../operations/load/k6.md)
- [Load testing](../operations/load/index.md)
