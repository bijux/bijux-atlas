# bijux-atlas-api

Deterministic API contract crate for `bijux-atlas`.

## Purpose
- Own OpenAPI generation and API wire contracts.
- Keep transport DTOs separate from internal model/query structs.
- Publish stable error and compatibility behavior.

## Stability
- Public entrypoints: `openapi_v1_spec`, request parsers, wire helpers.
- Compatibility and policy promises: [docs/API_STABILITY.md](docs/API_STABILITY.md).

## OpenAPI Workflow
- Generator: `cargo run -p bijux-atlas-api --bin atlas-openapi -- --out <path>`.
- Snapshot contract: `configs/openapi/v1/openapi.snapshot.json`.
- Drift guards: `tests/openapi_lint.rs` and `tests/params_validation.rs`.

## Do Not
- Do not expose internal domain structs in wire DTOs.
- Do not add ad-hoc per-handler error serialization.
- Do not emit non-deterministic OpenAPI output.

## Documentation
- [docs/INDEX.md](docs/INDEX.md)
