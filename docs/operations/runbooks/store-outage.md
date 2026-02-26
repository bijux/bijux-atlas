# Runbook: Store Outage

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-server`

## Symptoms

- Rising 5xx on uncached dataset opens.
- Dataset download failures.

## Metrics

- `bijux_store_download_p95_seconds`
- `bijux_store_breaker_open`
- `bijux_http_requests_total`

## Dashboard Panels

- `Store Open/Download p95`
- `Dataset Cache Hit/Miss`
- `HTTP Request Rate by Route/Status`

## Commands

```bash
$ make e2e-perf
$ curl -s http://127.0.0.1:8080/readyz
```

## Expected outputs

- `readyz` indicates degraded/not-ready when store is unavailable.
- Perf run shows cached-only behavior preserving cheap query availability.

## Mitigations

- Enable cached-only mode.
- Reduce heavy-query concurrency and strict limits.

## Alerts

- `BijuxAtlasStoreDownloadFailures`
- `BijuxAtlasCacheThrash`

## Rollback

- Restore store connectivity.
- Disable cached-only mode after stable metrics window.

## Postmortem checklist

- Incident timeline complete.
- Store dependency failure class identified.
- Retry/circuit-breaker thresholds adjusted if required.

## See also

- `ops-ci`

## Dashboards

- [Observability Dashboard](../observability/dashboard.md)

## Drills

- make ops-drill-store-outage
- make ops-drill-pod-churn
