# Atlas Contracts SSOT

`docs/contracts/` is the only source of truth for externally visible machine contracts.

## SSOT Files
- `ERROR_CODES.json`: stable machine error code registry.
- `METRICS.json`: required metrics and label schema.
- `ENDPOINTS.json`: API endpoint registry (`method`, `path`, telemetry class).
- `CHART_VALUES.json`: allowed Helm `values.yaml` top-level keys.
- `CLI_COMMANDS.json`: CLI command surface SSOT.

## Build Chain
1. `scripts/contracts/format_contracts.py` canonicalizes/sorts SSOT JSON.
2. `scripts/contracts/generate_contract_artifacts.py` generates:
   - `crates/bijux-atlas-api/src/generated/error_codes.rs`
   - `docs/contracts/generated/*.md`
   - `observability/metrics_contract.json` (derived compatibility artifact)
3. `scripts/openapi-generate.sh` builds OpenAPI and validates path set against `ENDPOINTS.json`.
4. `scripts/contracts/check_all.sh` enforces drift checks.

## Rule
- Surface changes must update SSOT first, then generated outputs, then code/tests.
