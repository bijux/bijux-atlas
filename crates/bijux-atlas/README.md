# bijux-atlas

`bijux-atlas` is the compatibility alias crate for
[`bijux-atlas-runtime`](https://crates.io/crates/bijux-atlas-runtime).

Use this crate when you want the durable `bijux_atlas` import path while relying
on the same canonical runtime implementation that now lives in
`bijux-atlas-runtime`.

## What It Does

- re-exports the public Rust API from `bijux-atlas-runtime`
- preserves module paths such as `bijux_atlas::query` and
  `bijux_atlas::adapters`
- stays subordinate to the canonical runtime crate instead of becoming a second
  implementation home
- keeps the public crate name short while internal workspace ownership stays
  explicit

## Compatibility Contract

If this works:

```rust
use bijux_atlas_runtime::query::Region;
```

the alias crate is expected to support the same import through:

```rust
use bijux_atlas::query::Region;
```

The runtime implementation, command dispatch, and binary ownership still belong
to the owning Atlas crates:

- `bijux-atlas-runtime`: canonical runtime library
- `bijux-atlas-cli`: `bijux-atlas` binary owner
- `bijux-atlas-server`: `bijux-atlas-server` binary owner
- `bijux-atlas-api`: `bijux-atlas-openapi` binary owner
