# Observability Contracts

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@31b823e9454cbf8762982048a3d5f0ef9098c3f5`
- Reason to exist: define the governed observability checks and the contract files they validate.

## Contract IDs

- `OBS-001`: required metrics are present in the baseline install metrics endpoint.
- `OBS-002`: warmup lock metrics are present and exposed.
- `OBS-003`: emitted API errors remain aligned with the governed error registry.
- `OBS-004`: startup logs include the required structured fields.
- `OBS-005`: observability evidence and collected logs do not expose secrets.

## Contract sources

- Log schema: `configs/contracts/observability/log.schema.json`
- Metrics schema: `configs/contracts/observability/metrics.schema.json`
- Error registry: `configs/contracts/observability/error-codes.json`
- Label policy: `configs/contracts/observability/label-policy.json`
- Verification report schema: `ops/schema/k8s/obs-verify.schema.json`
- Dashboard schema: `ops/schema/observe/dashboard.schema.json`
- PrometheusRule schema: `ops/schema/observe/prometheus-rule.schema.json`

## Validation entrypoint

Run `cargo run -q -p bijux-dev-atlas -- ops obs verify --allow-subprocess --allow-write --allow-network`.

Expected result: the command writes `ops-obs-verify.json`, validates the metrics endpoint, and confirms
the governed dashboard, alert, SLO, label, error, and log invariants.
