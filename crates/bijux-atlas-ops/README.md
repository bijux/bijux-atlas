# bijux-atlas-ops

Operational contract and surface ownership crate for `bijux-atlas`.

This crate owns durable references to repository surfaces that belong to stack,
ops, load, k8s, and generated operational assets. Higher-level tooling crates
such as `bijux-atlas-dev` should consume these surfaces instead of hard-coding
repository topology directly.

Current owned surfaces:

- `reference`: workspace-owned source and generated surface contracts.
- `kubernetes`: durable path contracts for Helm charts, values, toolchain
  inventory, rollout safety, and dataset manifests used by Atlas operations.
