# Metric Cardinality Guardrails

## Policy
- Metric names are versioned by contract in `ops/observe/contracts/metrics-contract.json`.
- Labels must be bounded and non-user-controlled.
- Forbidden labels include request-derived high-cardinality values such as `gene_id`, `name`, `cursor`, `region`, and raw `ip`.

## Allowed Dynamic Labels
- `route`
- `status`
- `query_type`
- `stage`
- `code`

## Enforcement
- `crates/bijux-dev-atlas/src/observability/contracts/metrics/check_metrics_contract.py` validates required metrics and label policy.
- `make observability-check` is the authoritative gate for observability contracts.

## See also

- `ops-ci`
