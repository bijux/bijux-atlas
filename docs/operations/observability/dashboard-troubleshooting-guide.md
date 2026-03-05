# Dashboard Troubleshooting Guide

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Common Issues

- Missing panels due to uid collision.
- Query errors from invalid metric names.
- Empty charts caused by missing scrape targets.

## Recovery Steps

1. Validate JSON against `ops/schema/observe/dashboard.schema.json`.
2. Check Prometheus target health for missing metrics.
3. Compare dashboard files against committed baseline.
4. Re-import corrected dashboard and confirm render status.
