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

## Artifacts
- `ops/_artifacts/<run_id>/load/`

## Failure modes
- Suite manifest/schema drift.
- Pinned query lock mismatch.
- Regression against accepted baseline thresholds.
