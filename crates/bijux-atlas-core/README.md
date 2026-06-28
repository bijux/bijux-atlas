# bijux-atlas-core

`bijux-atlas-core` owns runtime-independent Atlas primitives: canonical JSON
encoding, deterministic hashing, stable sorting helpers, and generated error
codes that other Atlas crates consume without pulling in runtime adapters.

Use this crate when you need:

- deterministic JSON or hashing helpers
- canonical cursor-payload encoding or decoding helpers
- stable key-based sorting primitives
- `Hash256` and related checksum utilities
- generated Atlas error-code definitions shared across crates

Public references:

- Project docs: <https://bijux.github.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-core/latest/bijux_atlas_core/>
- Source repository: <https://github.com/bijux/bijux-atlas>
