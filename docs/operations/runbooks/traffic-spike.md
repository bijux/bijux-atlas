# Traffic Spike Runbook

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Symptoms

- Rising `503` on non-cheap endpoints
- Increased queue depth and heavy-class saturation
- Elevated latency p95/p99

## Metrics

- `bijux_overload_shedding_active`
- `bijux_request_queue_depth`
- `bijux_class_heavy_inflight`
- `bijux_cheap_queries_served_while_overloaded_total`
- HTTP status split for `/v1/genes`, `/v1/sequence/region`, `/v1/transcripts/*`

## Dashboards

- `docs/operations/observability/INDEX.md` dashboard links
- Grafana via `make ops-open-grafana`

## Drill Steps

1. `make ops-drill-overload`
2. `make ops-load-shedding`
3. `make ops-drill-rate-limit`

## Expected Output

- Cheap-by-id requests remain available.
- Non-cheap/heavy requests may return controlled shed responses.
- Queue depth returns to baseline after load subsides.

## Mitigations

1. Enable/verify shedding policy and budgets.
2. Reduce heavy traffic sources or enforce stricter filters/range bounds.
3. Increase cheap/medium concurrency only if headroom exists.

## Rollback

1. Revert recent overload-policy config changes.
2. Re-run `make ops-load-shedding` to confirm recovery behavior.

## Postmortem Checklist

- Capture metrics snapshot and artifacts from `make ops-report`.
- Record trigger, blast radius, and time-to-recovery.
- Update threshold values only with benchmark evidence.
