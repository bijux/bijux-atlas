# bijux-atlas

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![bijux-atlas](https://img.shields.io/crates/v/bijux-atlas?label=bijux--atlas&logo=rust)](https://crates.io/crates/bijux-atlas)
[![rust-docs](https://img.shields.io/badge/rust--docs-bijux--atlas-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas/latest/bijux_atlas/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas` is the compatibility alias crate for the Atlas workspace.

Use this crate when you want the durable `bijux_atlas` import path while the
canonical implementation stays split across `bijux-atlas-runtime` and the leaf
owner crates.

## What It Does

- re-exports runtime-owned modules such as `bijux_atlas::adapters` and
  `bijux_atlas::runtime`
- preserves compatibility paths such as `bijux_atlas::query`,
  `bijux_atlas::api`, and `bijux_atlas::domain::ingest`
- keeps compatibility shims out of `bijux-atlas-runtime`, so the runtime crate
  does not pretend to own leaf crates
- keeps the public crate name short while internal workspace ownership stays
  explicit

## Compatibility Contract

If this works:

```rust
use bijux_atlas_query::Region;
```

the alias crate is expected to support the same import through:

```rust
use bijux_atlas::query::Region;
```

Command dispatch, HTTP delivery, ingest normalization, and query planning still
belong to the owning Atlas crates:

- `bijux-atlas-runtime`: canonical runtime library
- `bijux-atlas-cli`: `bijux-atlas` binary owner
- `bijux-atlas-server`: `bijux-atlas-server` binary owner
- `bijux-atlas-api`: `bijux-atlas-openapi` binary owner
- `bijux-atlas-query`: canonical query types and planning surface
- `bijux-atlas-ingest`: canonical ingest normalization and artifact building

## Choose This Crate When

- you are preserving an existing `bijux_atlas::...` import path
- you want one short compatibility dependency while the implementation remains
  split across owner crates
- you need compatibility without pretending the runtime crate owns every leaf
  surface

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas/latest/bijux_atlas/>
- Source repository: <https://github.com/bijux/bijux-atlas>
