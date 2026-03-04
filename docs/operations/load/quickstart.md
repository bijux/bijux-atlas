# Load Testing Quickstart

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Purpose

Provide the shortest path to execute a governed load suite and collect a report.

## Steps

1. Validate suite contracts:
   - `jq . ops/load/suites/suites.json > /dev/null`
2. Preview suite plan:
   - `bijux-dev-atlas ops load plan --suite mixed-workload --format json`
3. Run suite:
   - `bijux-dev-atlas ops load run --suite mixed-workload --allow-subprocess --allow-network --allow-write --format json`
4. Generate report:
   - `bijux-dev-atlas ops load report --suite mixed-workload --format json`

## Output

- Runtime report artifact: `artifacts/ops/<run_id>/load/mixed-workload/report.json`
