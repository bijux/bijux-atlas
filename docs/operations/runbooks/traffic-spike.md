# Traffic Spike Runbook

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Symptoms

- Rising `503` on non-cheap endpoints
- Increased queue depth and heavy-class saturation
- Elevated latency p95/p99

## Metrics

- `bijux_overload_shedding_active`
- `bijux_http_requests_total`
- `bijux_http_request_latency_p95_seconds`
- `bijux_errors_total`
- HTTP status split for `/v1/genes`, `/v1/sequence/region`, `/v1/trantooling/{tx_id}`

## Dashboards

- `docs/operations/observability/dashboard.md`
- Grafana via `make ops-open-grafana`

## Commands

1. `make ops-drill-overload`
2. `make ops-load-shedding`
3. `make ops-drill-rate-limit`

## Expected outputs

- Cheap-by-id requests remain available.
- Non-cheap/heavy requests may return controlled shed responses.
- Queue depth returns to baseline after load subsides.

## Mitigations

1. Enable/verify shedding policy and budgets.
2. Reduce heavy traffic sources or enforce stricter filters/range bounds.
3. Increase cheap/medium concurrency only if headroom exists.

## Alerts

- `AtlasOverloadSustained`
- `BijuxAtlasP95LatencyRegression`

## Rollback

1. Revert recent overload-policy config changes.
2. Re-run `make ops-load-shedding` to confirm recovery behavior.

## Postmortem checklist

- Capture metrics snapshot and artifacts from `make ops-report`.
- Record trigger, blast radius, and time-to-recovery.
- Update threshold values only with benchmark evidence.
