# Load Testing Strategy

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Purpose

Define the operational strategy for load testing Atlas before promotion and during regression triage.

## Strategy

1. Start with `mixed` as the release gate baseline.
2. Run scenario-specific suites for query-heavy, read-heavy, and write-heavy profiles.
3. Run stability and saturation suites in nightly lanes.
4. Compare output against committed baselines before adjusting thresholds.

## Dataset Scale

- `small`: smoke-level confidence checks.
- `medium`: release candidate verification.
- `large`: production-like stress and saturation checks.
- `x_large`: long-running and capacity boundary checks.

## Concurrency Levels

- `low`: 10-50 active virtual users.
- `medium`: 50-200 active virtual users.
- `high`: 200-500 active virtual users.
- `overload`: 500+ active virtual users for controlled degradation checks.
