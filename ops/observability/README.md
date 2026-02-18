# Ops Observability

Contract and drill pack for Atlas observability.

- Inputs: dashboard JSON, alert rules, metrics/tracing/log contracts, validation scripts.
- Outputs: validated observability assets and snapshots under `artifacts/ops/observability/`.
- Invariants: contract files are SSOT, dashboards/alerts are linted, traces/metrics snapshots are schema-checked.
- Gates: run `make ops-observability-validate` and `make ops-observability-pack-tests`.

Use make targets only; avoid direct script invocations in runbooks/docs.
Canonical ops entrypoint remains `ops/README.md`.
