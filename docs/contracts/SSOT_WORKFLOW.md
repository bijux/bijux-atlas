# How SSOT Works In Atlas

Atlas uses `docs/contracts/` as single-source-of-truth for machine contracts.

## Contracts
- `ERROR_CODES.json` -> generated Rust constants + OpenAPI enum validation.
- `METRICS.json` -> generated observability collector contract.
- `ENDPOINTS.json` -> route registry cross-check against server and OpenAPI.
- `CHART_VALUES.json` -> chart values key contract.
- `CLI_COMMANDS.json` -> CLI command list + docs consistency.

## Enforcement
- Local: `make ssot-check`
- CI: `ssot-drift` job in `.github/workflows/ci.yml`
- Policy lint includes SSOT gate.

## No Manual Drift
Generated files under `docs/contracts/generated/`, `crates/bijux-atlas-api/src/generated/`, and `observability/metrics_contract.json` must not be hand-edited.
