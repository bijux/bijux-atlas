# Minimal Release Surface

- Owner: `bijux-atlas-operations`
- Purpose: define minimum evidence and control documents required for release readiness.
- Consumers: `checks_ops_minimalism_and_deletion_safety`

## Minimal Release Surface

- `ops/inventory/contracts-map.json`
- `ops/inventory/authority-index.json`
- `ops/load/suites/suites.json`
- `ops/observe/drills/drills.json`
- `ops/report/generated/readiness-score.json`

## Deletion Impact Rules

Removing any listed item is a release blocker unless governance explicitly replaces the source and all dependent checks are updated in the same change.
