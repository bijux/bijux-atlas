# Ops Load

## Purpose
Own k6 suites/manifests, perf baselines, and load reporting.

## Entry points
- `make ops-load-suite SUITE=mixed PROFILE=kind`
- `make ops-load-smoke`
- `make ops-load-full`
- `make ops-load-manifest-validate`

## Contracts
- `ops/load/CONTRACT.md`
- `ops/load/suites/suites.json`
- `ops/load/contracts/deterministic-seed-policy.json`
- `ops/load/contracts/query-pack-catalog.json`

## Authored Inputs
- `ops/load/suites/suites.json`
- `ops/load/thresholds/*.thresholds.json`
- `ops/load/scenarios/*.json`
- `ops/load/k6/suites/*.js`

## Generated
- `ops/load/generated/suites.manifest.json`
- `ops/load/generated/load-summary.json`
- `ops/load/generated/load-drift-report.json`

## Artifacts
- `ops/_artifacts/<run_id>/load/`

## Failure modes
- Suite manifest/schema drift.
- Pinned query lock mismatch.
- Regression against accepted baseline thresholds.
