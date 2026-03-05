# Monitoring Troubleshooting Guide

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Symptoms

- Dashboard queries return no data.
- Panel rendering errors in Grafana.
- Missing metrics for expected services.

## Checks

1. Run `bijux-dev-atlas observe dashboards verify --format json`.
2. Validate scrape target health in Prometheus.
3. Confirm dashboard JSON files match registry paths.
4. Validate dashboard schema and metadata schema documents.
