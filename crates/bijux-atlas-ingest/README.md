# bijux-atlas-ingest

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-ingest)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![ingest](https://img.shields.io/crates/v/bijux-atlas-ingest?label=ingest&logo=rust)](https://crates.io/crates/bijux-atlas-ingest)
[![ghcr-ingest](https://img.shields.io/badge/ghcr-ingest-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-ingest)
[![rust-docs](https://img.shields.io/badge/rust--docs-ingest-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-ingest/latest/bijux_atlas_ingest/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-ingest` is the published library crate that owns the path from
governed source files to build-ready Atlas artifacts. It is where raw GFF3 and
FASTA inputs stop being source files and start becoming deterministic release
material.

## Choose This Crate When

- deterministic Atlas ingest execution
- ingest artifact and anomaly report generation
- ingest normalization replay and diff support
- ingest-owned tests and benchmark surfaces

## What It Does Not Own

This crate does not own query execution, artifact publication, HTTP serving, or
CLI process wiring. It owns the ingest boundary itself and is then composed by
`bijux-atlas-runtime`.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-ingest/latest/bijux_atlas_ingest/>
- Source repository: <https://github.com/bijux/bijux-atlas>
