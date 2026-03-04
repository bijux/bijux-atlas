# Load Architecture Diagram

- Owner: `bijux-atlas-operations`
- Type: `concept`
- Audience: `operator`
- Stability: `stable`

## Purpose

Describe load execution flow from suite definition to report artifacts.

```mermaid
flowchart LR
    A[ops/load/suites/suites.json] --> B[ops load plan]
    B --> C[ops load run]
    C --> D[k6 suite script]
    D --> E[k6 summary output]
    E --> F[ops load report]
    F --> G[artifacts/ops/<run_id>/load/<suite>/report.json]
    A --> H[ops/load/contracts/k6-thresholds.v1.json]
    H --> F
```
