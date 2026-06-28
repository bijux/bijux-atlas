# bijux-atlas-api

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-api)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![api](https://img.shields.io/crates/v/bijux-atlas-api?label=api&logo=rust)](https://crates.io/crates/bijux-atlas-api)
[![rust-docs](https://img.shields.io/badge/rust--docs-api-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-api/latest/bijux_atlas_api/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-api` owns the stable Atlas API boundary: request parameter
parsing, response DTOs, compatibility aliases, and OpenAPI generation that
runtime adapters expose but do not redefine.

## What This Crate Owns

- request parameter parsing and validation
- response DTO and error envelope contracts
- OpenAPI v1 document generation
- compatibility redirects and stable API error-code aliases
- crate-owned API contract, HTTP surface, and observability test coverage
- OpenAPI benchmark coverage and compatibility guards

## What It Does Not Own

- HTTP process startup and listener lifecycle, owned by `bijux-atlas-server`
- dataset ingest, query execution, or artifact publication, owned by the leaf
  product crates and composed by `bijux-atlas-runtime`
- repository governance automation, owned by `bijux-atlas-dev`

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-api/latest/bijux_atlas_api/>
- Source repository: <https://github.com/bijux/bijux-atlas>
- Crate boundary reference: <https://bijux.io/bijux-atlas/bijux-atlas/foundations/crate-boundary-contract/>
