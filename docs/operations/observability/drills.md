# Observability Drills

- Owner: `bijux-atlas-operations`

## Contract

- Drill manifest SSOT: `ops/observability/drills/drills.json`
- Drill result schema: `ops/observability/drills/result.schema.json`
- Runner: `scripts/run_drill.sh <name>`
- Suite target: `make ops-drill-suite`

## Reproducibility rules

- Deterministic timeouts per drill (`timeout_seconds` in manifest).
- No random sleeps or jitter in drill orchestration.
- Warmup and cleanup flags are explicit per drill.
- Parallelism policy is explicit (`parallel_policy`).

## Outputs

- Per-drill result: `artifacts/observability/drills/<name>.result.json`
- Suite report: `artifacts/observability/drill-conformance-report.json`

## Nightly

- Workflow: `.github/workflows/drill-suite-nightly.yml`
