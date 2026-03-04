# System Simulation Results

This page documents where deterministic simulation evidence is produced and how to review it.

## Output Artifacts

- `artifacts/system/simulation/index.json`
- `artifacts/system/simulation/coverage.json`
- `artifacts/system/simulation/resilience-report.json`
- `artifacts/system/simulation/slo-validation.json`
- `artifacts/system/simulation/dashboard.md`

Each scenario also emits:

- `summary.json`
- `summary.md`
- `logs.json`
- `rendered-manifests.json`
- `health-checks.json`
- `event-timeline.json`
- `evidence-bundle.json`

## Scenarios

Scenarios are defined in `configs/system/simulation-scenarios.json` and executed in deterministic order by `scenario.id`.

## CI

The `system-simulation` workflow runs the suite and uploads simulation artifacts on pull requests and manual dispatch.
