# Docs Drift Dashboard

- Owner: `bijux-atlas-platform`
- Purpose: track orphan docs, ownership gaps, and stale generated metadata.

## Primary Inputs

- `docs/_internal/generated/docs-quality-dashboard.json`
- `docs/_internal/generated/docs-inventory.md`
- `configs/inventory/docs-owners.json`

## Signals

- Orphan docs warnings.
- Missing ownership mappings.
- Broken link and nav consistency regressions.
