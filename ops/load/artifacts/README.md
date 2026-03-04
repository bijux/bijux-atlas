# Load Artifact Storage

- Owner: `bijux-atlas-performance`
- Purpose: `define durable storage locations for load run artifacts`

## Paths

- Runtime artifacts: `artifacts/ops/<run_id>/load/<suite>/`
- Generated summary: `ops/load/generated/load-summary.json`
- Generated drift report: `ops/load/generated/load-drift-report.json`
- Baseline references: `ops/load/baselines/`

## Retention

- Keep baseline and generated contract artifacts under version control.
- Keep runtime artifacts in CI artifacts or object storage; do not commit runtime raw outputs.
