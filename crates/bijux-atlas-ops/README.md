# bijux-atlas-ops

Operational contract and surface ownership crate for `bijux-atlas`.

This crate is repository-owned support infrastructure. It exists to keep stack,
Kubernetes, load, observability, release, and operational path contracts out of
the maintainer control plane and out of product-facing runtime crates. It is
published to crates.io in the `0.2.2` release line so operations-contract
consumers can depend on the same owned path and stack surface registry that the
Atlas workspace uses internally.

Public references:

- Project docs: <https://bijux.io/bijux-atlas/>
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
