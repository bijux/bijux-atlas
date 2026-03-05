# Reproducibility Command Reference

- `bijux-dev-atlas reproduce run`
  - Builds a reproducibility run payload and writes `run-report.json`.
- `bijux-dev-atlas reproduce verify`
  - Verifies required scenarios and deterministic output shape.
- `bijux-dev-atlas reproduce explain [scenario]`
  - Shows one scenario or the full catalog.
- `bijux-dev-atlas reproduce status`
  - Returns current verification status and evidence presence.
- `bijux-dev-atlas reproduce audit-report`
  - Writes and emits reproducibility audit summary JSON.
- `bijux-dev-atlas reproduce metrics`
  - Writes and emits reproducibility metrics JSON.
- `bijux-dev-atlas reproduce lineage-validate`
  - Validates required lineage artifacts are hash-covered.
- `bijux-dev-atlas reproduce summary-table`
  - Generates markdown summary table for scenarios.
