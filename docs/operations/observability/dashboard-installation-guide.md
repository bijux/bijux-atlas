# Dashboard Installation Guide

- Owner: `bijux-atlas-operations`
- Type: `guide`
- Audience: `operator`
- Stability: `stable`

## Install

1. Import JSON files from `ops/observe/dashboards/`.
2. Verify each dashboard uid is unique.
3. Bind Prometheus datasource for all query panels.
4. Run dashboard schema validation contract.

## Verify

- Panels render without datasource errors.
- Core views include runtime, query, ingest, registry, resources, and SLO.
