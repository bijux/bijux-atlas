# Observability Drills

- Owner: `bijux-atlas-operations`

## Contract

- Drill manifest SSOT: `ops/observe/drills/drills.json`
- Drill result schema: `ops/observe/drills/result.schema.json`
- Runner: `bijux dev atlas ops obs drill run <name>`
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
