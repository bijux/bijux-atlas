# Atlas Contracts SSOT

`docs/contracts/` is the only source of truth for externally visible machine contracts.

## SSOT Files
- `ERROR_CODES.json`: stable machine error code registry.
- `METRICS.json`: required metrics and label schema.
- `TRACE_SPANS.json`: required trace span names + required attributes.
- `ENDPOINTS.json`: API endpoint registry (`method`, `path`, telemetry class).
- `CHART_VALUES.json`: allowed Helm `values.yaml` top-level keys.
- `CLI_COMMANDS.json`: CLI command surface SSOT.
- `CONFIG_KEYS.json`: allowed environment/config keys read by runtime and tools.
- `ARTIFACT_SCHEMA.json`: manifest/QC/db-meta contract for dataset artifacts.
- `POLICY_SCHEMA.json`: SSOT mirror of `configs/policy/policy.schema.json`.

## Build Chain
1. `scripts/contracts/format_contracts.py` canonicalizes/sorts SSOT JSON.
2. `scripts/contracts/generate_contract_artifacts.py` generates:
   - `crates/bijux-atlas-api/src/generated/error_codes.rs`
   - `crates/bijux-atlas-server/src/telemetry/generated/*.rs`
   - `docs/contracts/generated/*.md`
   - `observability/metrics_contract.json` (derived compatibility artifact)
3. `scripts/openapi-generate.sh` builds OpenAPI and validates path set against `ENDPOINTS.json`.
4. `scripts/contracts/check_all.sh` enforces drift checks, config-key contract, and breaking-change detection against previous `v*` tag.

## Convenience Runner
- `cargo run --manifest-path xtask/Cargo.toml -- format-contracts`
- `cargo run --manifest-path xtask/Cargo.toml -- generate-contracts`
- `cargo run --manifest-path xtask/Cargo.toml -- check-contracts`

## Rule
- Surface changes must update SSOT first, then generated outputs, then code/tests.
