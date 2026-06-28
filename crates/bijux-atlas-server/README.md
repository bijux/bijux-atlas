# bijux-atlas-server

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-server)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![server](https://img.shields.io/crates/v/bijux-atlas-server?label=server&logo=rust)](https://crates.io/crates/bijux-atlas-server)
[![rust-docs](https://img.shields.io/badge/rust--docs-server-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-server/latest/bijux_atlas_server/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-server` owns the long-running `bijux-atlas-server` executable.

This crate is the operator-facing binary-owner surface for Atlas HTTP serving,
runtime configuration loading, telemetry startup, and cache warmup behavior.
The reusable runtime implementation lives in the canonical
`bijux-atlas-runtime` library crate, while `bijux-atlas` remains the
compatibility alias for the historical import path. This package owns the
deployed server process contract.

## What This Crate Owns

- the installed `bijux-atlas-server` binary
- server process startup and shutdown wiring
- runtime config loading for the HTTP process
- telemetry bootstrap, route exposure, and cache warmup behavior
- server-facing tests and benchmarks that validate the deployed process surface

## Install and Verify

```bash
cargo install --locked bijux-atlas-server --bin bijux-atlas-server
bijux-atlas-server --help
```

## What It Does Not Own

- end-user CLI command ownership, owned by `bijux-atlas-cli`
- OpenAPI export ownership, owned by `bijux-atlas-api`
- leaf query, ingest, and store implementations, composed by
  `bijux-atlas-runtime`

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-server/latest/bijux_atlas_server/>
- Source repository: <https://github.com/bijux/bijux-atlas>
