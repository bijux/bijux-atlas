# Ops Drift Dashboard

- Owner: `bijux-atlas-operations`
- Purpose: track ops doc ownership drift and orphan specification files.

## Primary Inputs

- `ops/_generated.example/orphan-files-report.json`
- `ops/_generated.example/file-usage-report.json`
- `configs/inventory/ops-owners.json`

## Signals

- Unmapped ops files.
- Ownership coverage drift.
- Stale or unused ops markdown surfaces.
