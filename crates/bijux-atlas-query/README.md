# bijux-atlas-query

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-query)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![query](https://img.shields.io/crates/v/bijux-atlas-query?label=query&logo=rust)](https://crates.io/crates/bijux-atlas-query)
[![ghcr-query](https://img.shields.io/badge/ghcr-query-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-query)
[![rust-docs](https://img.shields.io/badge/rust--docs-query-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-query/latest/bijux_atlas_query/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-query` is the published library crate that owns the Atlas query
language and execution boundary. It is where requests become plans, cursors,
and SQLite-backed result sets with stable semantics.

## Choose This Crate When

- gene and transcript query request parsing
- deterministic query-plan classification and budgeting
- cursor encode or decode helpers for pagination
- SQLite-backed query execution and explain-plan inspection
- owned query benches and fixture contracts

It depends on `bijux-atlas-core` for canonical hashing and on
`bijux-atlas-model` for shared dataset, diff, and gene value types.

## What It Does Not Own

`bijux-atlas-query` owns the query boundary itself, but not CLI presentation,
HTTP lifecycle, ingest normalization, or artifact publication.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-query/latest/bijux_atlas_query/>
- Source repository: <https://github.com/bijux/bijux-atlas>
