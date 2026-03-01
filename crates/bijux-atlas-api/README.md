# bijux-atlas-api

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Deterministic API contract crate for `bijux-atlas`.

## Purpose
- Own OpenAPI generation and API wire contracts.
- Keep transport DTOs separate from internal model/query structs.
- Publish stable error and compatibility behavior.

## Stability
- Public entrypoints: `openapi_v1_spec`, request parsers, wire helpers.
- Compatibility and policy promises: [docs/api-stability.md](docs/api-stability.md).

## OpenAPI Workflow
- Generator: `cargo run -p bijux-atlas-api --bin atlas-openapi -- --out <path>`.
- Snapshot contract: `configs/openapi/v1/openapi.snapshot.json`.
- Drift guards: `tests/openapi_lint.rs` and `tests/params_validation.rs`.

## Do Not
- Do not expose internal domain structs in wire DTOs.
- Do not add ad-hoc per-handler error serialization.
- Do not emit non-deterministic OpenAPI output.

## Documentation
- [docs/index.md](docs/index.md)
