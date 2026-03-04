# Load Testing Summary Report

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`

## Purpose

Define the system load summary report used for release and regression review.

## Artifact

- `ops/load/generated/system-load-summary.json`

## Required Fields

- `schema_version`
- `kind`
- `status`
- `baseline_profile`
- `candidate_profile`
- `summary.suites_total`
- `summary.regressions`
- `summary.notes`
