# Performance Architecture Diagrams

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@2228f79ef`

## Purpose

Describe performance governance data flow from suite execution to CI and operator evidence.

```mermaid
flowchart LR
    A[ops/load/suites/suites.json] --> B[ops load run]
    B --> C[k6 summary]
    C --> D[regression detector]
    D --> E[regression report]
    E --> F[history and trend assets]
    F --> G[dashboard and badges]
    G --> H[performance regression CI]
```
