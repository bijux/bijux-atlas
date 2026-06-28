# bijux-atlas-cli

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![cli](https://img.shields.io/crates/v/bijux-atlas-cli?label=cli&logo=rust)](https://crates.io/crates/bijux-atlas-cli)
[![ghcr-cli](https://img.shields.io/badge/ghcr-cli-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-cli)
[![rust-docs](https://img.shields.io/badge/rust--docs-cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-cli/latest/bijux_atlas_cli/)
[![docs-atlas](https://img.shields.io/badge/docs-atlas-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas/)

`bijux-atlas-cli` owns the installed `bijux-atlas` command. It is the direct
user-facing entrypoint for browsing datasets, validating inputs, running
ingest flows, exporting OpenAPI, and inspecting release state.

## What This Crate Owns

- the installed `bijux-atlas` binary
- CLI argument parsing and noun-first command dispatch
- direct CLI tests that validate the published command contract
- the boundary between end-user commands and runtime orchestration

## Choose This Crate When

- you want the Atlas command-line surface exactly as users install it
- you are extending CLI nouns, flags, or structured output expectations
- you need the direct Cargo-managed Atlas binary for local use or CI

## Install and Verify

```bash
cargo install --locked bijux-atlas-cli --bin bijux-atlas
bijux-atlas --help
bijux-atlas version
```

## What It Does Not Own

- long-running HTTP process behavior, owned by `bijux-atlas-server`
- OpenAPI export ownership, owned by `bijux-atlas-api`
- runtime orchestration internals, owned by `bijux-atlas-runtime`

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-cli/latest/bijux_atlas_cli/>
- Source repository: <https://github.com/bijux/bijux-atlas>
