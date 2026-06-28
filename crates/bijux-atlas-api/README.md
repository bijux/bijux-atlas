# bijux-atlas-api

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-api)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![api](https://img.shields.io/crates/v/bijux-atlas-api?label=api&logo=rust)](https://crates.io/crates/bijux-atlas-api)
[![ghcr-api](https://img.shields.io/badge/ghcr-api-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-api)
[![rust-docs](https://img.shields.io/badge/rust--docs-api-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-api/latest/bijux_atlas_api/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-api` is the contract crate for Atlas HTTP and OpenAPI consumers.
It owns the request, response, error, and schema surfaces that the rest of the
runtime is required to honor.

## What This Crate Owns

- request parameter parsing and validation
- response DTOs and error-envelope contracts
- OpenAPI document generation and compatibility aliases
- API-facing tests and observability checks that keep the wire surface stable

## Choose This Crate When

- you need Atlas request or response types in another Rust crate
- you want the owned OpenAPI export surface instead of scraping runtime routes
- you need the strongest compatibility story for API envelopes and parameters

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
