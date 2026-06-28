# bijux-atlas-core

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-core)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![core](https://img.shields.io/crates/v/bijux-atlas-core?label=core&logo=rust)](https://crates.io/crates/bijux-atlas-core)
[![ghcr-core](https://img.shields.io/badge/ghcr-core-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-core)
[![rust-docs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-core/latest/bijux_atlas_core/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-core` is the smallest published library layer in the Atlas
workspace. It holds deterministic primitives that should remain useful without
bringing in runtime policy, adapters, binaries, or storage behavior.

## Choose This Crate When

- deterministic JSON or hashing helpers
- canonical cursor-payload encoding or decoding helpers
- stable key-based sorting primitives
- `Hash256` and related checksum utilities
- generated Atlas error-code definitions shared across crates

## What It Does Not Own

`bijux-atlas-core` does not own dataset semantics, query planning, ingest
execution, storage backends, or runtime wiring. Those responsibilities stay in
the higher-level Atlas crates so core can remain runtime-independent and widely
reusable.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-core/latest/bijux_atlas_core/>
- Source repository: <https://github.com/bijux/bijux-atlas>
