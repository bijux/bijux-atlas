# Load Harness And Generators

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Purpose

Document the load harness and request generators used by Atlas load suites.

## Harness

- Manifest: `ops/load/suites/suites.json`
- Scenario catalog: `ops/load/scenarios/`
- Runtime integration: `bijux-dev-atlas ops load plan|run|report`
- Result artifacts: `artifacts/ops/<run_id>/load/<suite>/`

## Load Generators

- HTTP generator via k6 scripts in `ops/load/k6/suites/`
- Query-oriented generator through query endpoint scenarios
- Ingest-oriented generator through ingest endpoint scenarios

Concrete reusable generator scripts:

- `ops/load/k6/suites/http-request-generator.js`
- `ops/load/k6/suites/query-load-generator.js`
- `ops/load/k6/suites/ingest-load-generator.js`

## Commands

```bash
bijux-dev-atlas ops load plan --suite mixed --format json
bijux-dev-atlas ops load run --suite mixed --allow-subprocess --allow-network --allow-write --format json
bijux-dev-atlas ops load report --suite mixed --format json
```
