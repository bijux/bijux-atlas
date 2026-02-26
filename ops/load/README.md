# Ops Load

Load suites, thresholds, scenarios, and generated reporting artifacts.

## Start Here
- `ops/load/CONTRACT.md`
- `ops/load/suites/suites.json`
- `ops/load/suites/suites.json`
- `ops/load/thresholds/*.thresholds.json`
- `ops/load/scenarios/*.json`
- `ops/load/k6/suites/*.js`
- `ops/load/contracts/deterministic-seed-policy.json`

## Generated
- `ops/load/generated/suites.manifest.json`
- `ops/load/generated/load-summary.json`
- `ops/load/generated/load-drift-report.json`

## Entrypoints
- `make ops-load-suite SUITE=mixed PROFILE=kind`
- `make ops-load-smoke`
- `make ops-load-full`

Placeholder extension directories tracked with `.gitkeep`: `ops/load/data`, `ops/load/thresholds`, `ops/load/k6/thresholds`.
