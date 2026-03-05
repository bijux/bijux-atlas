# Observability Architecture Diagram

```mermaid
flowchart LR
  A[Atlas Runtime] --> B[/metrics]
  A --> C[Structured Logs]
  A --> D[Traces]
  A --> E[/healthz /readyz /debug/*]
  B --> F[Prometheus]
  D --> I[Trace Backend]
  C --> H[Log Store]
  F --> G[Grafana]
  I --> G
  H --> G
  G --> K[Runtime Dashboard]
  G --> L[Query Dashboard]
  G --> M[Ingest Dashboard]
  G --> N[Registry Dashboard]
  G --> O[SLO Dashboard]
  E --> J[Operational CLI]
  J --> P[Readiness And Telemetry Artifacts]
```
