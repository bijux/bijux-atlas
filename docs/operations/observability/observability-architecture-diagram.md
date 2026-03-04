# Observability Architecture Diagram

```mermaid
flowchart LR
  A[Atlas Runtime] --> B[/metrics]
  A --> C[Structured Logs]
  A --> D[Traces]
  A --> E[/healthz /readyz /debug/*]
  B --> F[Prometheus]
  F --> G[Grafana]
  C --> H[Log Store]
  H --> G
  D --> I[Trace Backend]
  I --> G
  E --> J[System Debug CLI]
```
