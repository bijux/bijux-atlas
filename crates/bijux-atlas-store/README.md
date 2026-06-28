# bijux-atlas-store

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-store)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![store](https://img.shields.io/crates/v/bijux-atlas-store?label=store&logo=rust)](https://crates.io/crates/bijux-atlas-store)
[![ghcr-store](https://img.shields.io/badge/ghcr-store-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-store)
[![rust-docs](https://img.shields.io/badge/rust--docs-store-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-store/latest/bijux_atlas_store/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-store` owns Atlas publication and storage semantics. It defines
how artifacts are laid out, locked, verified, and persisted across local and
remote backends.

## Choose This Crate When

- publish-time `ArtifactStore` contracts
- deterministic dataset artifact path and key layout
- manifest lock creation and checksum verification
- local filesystem, HTTP readonly, or S3-like artifact backends
- owned storage benches and infrastructure tests

## What It Does Not Own

This crate does not own ingest normalization, query planning, CLI dispatch, or
HTTP process behavior. It owns immutable artifact publication and backend
verification semantics.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-store/latest/bijux_atlas_store/>
- Source repository: <https://github.com/bijux/bijux-atlas>
