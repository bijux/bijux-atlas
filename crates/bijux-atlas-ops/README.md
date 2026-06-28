# bijux-atlas-ops

[![Rust 1.86+](https://img.shields.io/badge/rust-1.86%2B-DEA584?logo=rust&logoColor=white)](https://crates.io/crates/bijux-atlas-ops)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-atlas/blob/main/LICENSE)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--atlas-181717?logo=github)](https://github.com/bijux/bijux-atlas)
[![ops](https://img.shields.io/crates/v/bijux-atlas-ops?label=ops&logo=rust)](https://crates.io/crates/bijux-atlas-ops)
[![ghcr-ops](https://img.shields.io/badge/ghcr-ops-181717?logo=github)](https://github.com/bijux/bijux-atlas/pkgs/container/bijux-atlas%2Fbijux-atlas-ops)
[![rust-docs](https://img.shields.io/badge/rust--docs-ops-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-atlas-ops/latest/bijux_atlas_ops/)
[![docs-operations](https://img.shields.io/badge/docs-operations-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-atlas/bijux-atlas-ops/)

`bijux-atlas-ops` is the published operations-contract library crate for
Atlas. It holds the durable references and owned metadata that operators,
release tools, and maintainer automation need for stack, Kubernetes, load,
observability, and release-support surfaces.

It does not install an end-user command or a server process by itself. Its
value is the contract surface that other Atlas crates and release workflows can
consume without hard-coding repository topology.

## What This Crate Owns

- operational path contracts and reference surfaces
- Kubernetes and Helm ownership metadata
- observability, load, and release-support contract fixtures
- reusable repository-owned ops references consumed by higher-level tooling

## Choose This Crate When

- you need the owned Atlas operations surface in another Rust crate
- you want operator-facing paths and references without hard-coding repository
  topology
- you are extending stack, load, release, or observability contracts

## What It Does Not Own

`bijux-atlas-ops` does not own runtime command dispatch, HTTP serving, dataset
build logic, or repository governance orchestration. Those surfaces stay in
`bijux-atlas-cli`, `bijux-atlas-server`, the product leaf crates, and
`bijux-atlas-dev`.

## Documentation

- Atlas handbook: <https://bijux.io/bijux-atlas/>
- Rust API docs: <https://docs.rs/bijux-atlas-ops/latest/bijux_atlas_ops/>
- Source repository: <https://github.com/bijux/bijux-atlas>

This crate owns durable references to repository surfaces that belong to stack,
ops, load, k8s, and generated operational assets. Higher-level tooling crates
such as `bijux-atlas-dev` should consume these surfaces instead of hard-coding
repository topology directly.

Current owned surfaces:

- `reference`: workspace-owned source and generated surface contracts.
- `kubernetes`: durable path contracts for Helm charts, values, toolchain
  inventory, rollout safety, and dataset manifests used by Atlas operations.
