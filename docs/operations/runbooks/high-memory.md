# Runbook: High Memory

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-server`

## Symptoms

- RSS growth crossing memory budget.
- OOM restarts under sustained load.

## Metrics

- `bijux_dataset_disk_usage_bytes`
- `bijux_overload_shedding_active`
- `bijux_http_request_latency_p95_seconds`

## Commands

```bash
$ make e2e-perf
$ cargo test -p bijux-atlas-server --test p99-regression
```

## Expected outputs

- Perf summary shows sustained memory pressure signature.
- Latency guard remains within target when mitigations applied.

## Mitigations

- Reduce cache budgets and open-shard caps.
- Lower heavy concurrency and tighten response-size limits.

## Alerts

- `BijuxAtlasP95LatencyRegression`

## Rollback

- Revert recent performance-related config change.
- Scale replicas while investigating memory profile.

## Postmortem checklist

- Memory hotspots documented.
- Capacity model updated.
- Guardrails tuned and tested.

## See also

- `ops-ci`

## Dashboards

- [Observability Dashboard](../observability/dashboard.md)

## Drills

- make ops-drill-store-outage
- make ops-drill-pod-churn
