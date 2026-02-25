# Changelog

All notable changes are documented in this file.

## v0.1.0

### Added
- Deterministic Rust workspace split between runtime (`bijux-atlas`) and control plane (`bijux-dev-atlas`).
- Canonical command surfaces for atlas runtime and dev control-plane workflows.
- SSOT governance for ops/config/docs policies and contracts.
- Registry-driven documentation inventory, validation, and generated indexes.
- Structured reports and CI lanes for check, lint, audit, test, docs, and ops validation.
- Apache-2.0 licensing metadata across workspace crates.

### Included Surfaces
- `ops/`: stack, render, validation, k8s and load orchestration contracts.
- `configs/`: policy/config schema and inventory contracts.
- `makefiles/`: thin wrapper targets around canonical crate commands.

### Notes
- `v0.1.0` establishes the stable baseline for command, contract, and policy governance.
