# Runbook: Memory Profile Under Load

- Owner: `bijux-atlas-server`

## Symptoms

- Memory growth uncorrelated with request volume.

## Metrics

- `bijux_dataset_disk_usage_bytes`
- `bijux_http_request_latency_p95_seconds`
- `bijux_sqlite_query_latency_p95_seconds`

## Commands

```bash
$ k6 run load/k6/atlas_1000qps.js
$ make e2e-perf
```

## Expected outputs

- Profiling captures stable allocation hot paths.
- No unbounded growth during steady-state window.

## Mitigations

- Reduce cache retention and response payload size.
- Apply allocation hot-path fixes.

## Rollback

- Disable recent memory-optimization experiment if regressions appear.

## Postmortem checklist

- Profile artifacts stored in artifacts path.
- Allocation diff documented.
- Next benchmark baseline updated.
