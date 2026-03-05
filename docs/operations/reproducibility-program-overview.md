# Reproducibility Program Overview

This program defines how Atlas proves release artifacts can be reproduced and validated with deterministic outputs.

Core command flow:
1. `bijux-dev-atlas reproduce run`
2. `bijux-dev-atlas reproduce verify`
3. `bijux-dev-atlas reproduce lineage-validate`
4. `bijux-dev-atlas reproduce audit-report`
5. `bijux-dev-atlas reproduce summary-table`

Primary inputs:
- `ops/reproducibility/spec.json`
- `ops/reproducibility/scenarios.json`
- `ops/reproducibility/report.schema.json`

Primary evidence outputs:
- `artifacts/reproducibility/run-report.json`
- `artifacts/reproducibility/audit-report.json`
- `artifacts/reproducibility/metrics.json`
- `artifacts/reproducibility/summary-table.md`
