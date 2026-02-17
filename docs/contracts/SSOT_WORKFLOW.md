# How SSOT Works In Atlas

Atlas uses `docs/contracts/` as single-source-of-truth for machine contracts.

## Contracts
- `ERROR_CODES.json` -> generated Rust constants + OpenAPI enum validation.
- `METRICS.json` -> generated observability collector contract.
- `TRACE_SPANS.json` -> generated trace span name/attributes contract.
- `ENDPOINTS.json` -> route registry cross-check against server and OpenAPI.
- `CHART_VALUES.json` -> chart values key contract.
- `CLI_COMMANDS.json` -> CLI command list + docs consistency.
- `CONFIG_KEYS.json` -> config/env key allowlist contract.
- `ARTIFACT_SCHEMA.json` -> artifact manifest/QC/db-meta schema contract.

## Enforcement
- Local: `make ssot-check`
- CI: `ssot-drift` job in `.github/workflows/ci.yml`
- Policy lint includes SSOT gate.
- Compatibility guard: `scripts/contracts/check_breaking_contract_change.py` compares against latest `v*` tag.

## No Manual Drift
Generated files under `docs/contracts/generated/`, `crates/bijux-atlas-api/src/generated/`, `crates/bijux-atlas-server/src/telemetry/generated/`, and `observability/metrics_contract.json` must not be hand-edited.
