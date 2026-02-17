# Runbook: Profile Under Load

- Owner: `bijux-atlas-server`

## Symptoms

- CPU saturation and latency spikes under reproducible load.

## Metrics

- `bijux_http_request_latency_p95_seconds`
- `bijux_sqlite_query_latency_p95_seconds`
- `bijux_request_stage_latency_p95_seconds`

## Commands

```bash
$ k6 run ops/load/k6/atlas_phase11.js
$ make e2e-perf
```

## Expected outputs

- Flamegraph or profiler output identifies dominant hot paths.
- Stage latency metrics correlate to measured hotspots.

## Mitigations

- Optimize query planner/projection for hot requests.
- Tune concurrency classes and statement caching.

## Rollback

- Revert optimization change with negative p99 impact.

## Postmortem checklist

- Perf diff report attached.
- Regression root cause documented.
- Preventive test added.
