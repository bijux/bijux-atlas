# bijux-atlas

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
