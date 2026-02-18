# Observability Contracts

## Inputs

- `ops/observability/contract/metrics-contract.json`
- `ops/observability/contract/alerts-contract.json`
- `ops/observability/contract/dashboard-panels-contract.json`
- `ops/observability/contract/logs-fields-contract.json`

## Outputs

- Contract validation status from `make ops-observability-validate`
- Metrics/traces snapshots in `artifacts/ops/observability/`
- Contract test results from `make ops-observability-pack-tests`

## Invariants

- Contract JSON files under `ops/observability/contract/` are SSOT.
- Dashboard and alert assets must pass contract lint checks.
- Runtime snapshots must satisfy metrics/tracing/log schemas.

## Gates

- `make ops-dashboards-validate`
- `make ops-alerts-validate`
- `make ops-observability-validate`
- `make ops-observability-pack-tests`

Do not duplicate contract files at `ops/observability/` root.
