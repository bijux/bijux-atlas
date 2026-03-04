# Operational Troubleshooting Guide

- Start with `/ready` and `/healthz/overload` to classify readiness vs saturation.
- Pull `/debug/diagnostics` and `/debug/runtime-stats` for queue depth and request pressure.
- Pull `/debug/cache-stats`, `/debug/dataset-registry`, and `/debug/shard-map` for dataset routing and cache churn.
- Pull `/debug/query-planner-stats` when latency or rejection rates spike.
- Capture `metrics` and structured logs for the same incident window.

## Evidence order

1. Health and readiness snapshots.
2. Runtime and planner diagnostics.
3. Cache and shard diagnostics.
4. Metrics slice and correlated logs.
