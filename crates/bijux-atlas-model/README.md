# bijux-atlas-model

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-model)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![model](https://img.shields.io/crates/v/bijux-atlas-model?label=model&logo=rust)](https://crates.io/crates/bijux-atlas-model)
[![ghcr-model](https://img.shields.io/badge/ghcr-model-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-model)
[![rust-docs](https://img.shields.io/badge/rust--docs-model-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-model/latest/bijux_atlas_model/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-model` is the published durable type boundary for Atlas. It owns
the value objects that should mean the same thing across ingest, query,
storage, API, and compatibility surfaces.

## Choose This Crate When

- dataset ids, catalogs, manifests, and shard catalogs
- gene, transcript, seqid, and region model types
- release diff payloads and release gene index records
- policy value objects that should stay outside runtime adapters

## What It Does Not Own

`bijux-atlas-model` is a type boundary, not a runtime or storage boundary. It
does not own query engines, ingest normalization, HTTP adapters, or artifact
backends.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-model/latest/bijux_atlas_model/>
- Source repository: <https://github.com/bijux/bijux-atlas>
