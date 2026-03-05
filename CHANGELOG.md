# Changelog

## v0.1.2

### Added
- Ops purity governance reports: `artifacts/governance/ops_purity_report.json` and `artifacts/governance/ops_purity_report.md`.

### Changed
- `migrations status` now fails when Python files are present under `ops/`.
- Governance report now includes `Directory Purity` and `Repo purity` sections.

### Fixed
- Removed `ops/cli/perf/cli_ux_benchmark.py`; CLI UX benchmark execution is now dev-atlas only.

### Breaking Changes
- none


## v0.1.1

### Added
- 

### Changed
- 

### Fixed
- 

### Breaking Changes
- none


All notable changes are documented in this file.

## v0.1.0

### Added
- Deterministic Rust workspace split between runtime (`bijux-atlas`) and control plane (`bijux-dev-atlas`).
- Canonical command surfaces for atlas runtime and dev control-plane workflows.
- SSOT governance for ops/config/docs policies and contracts.
- Registry-driven documentation inventory, validation, and generated indexes.
- Structured reports and CI lanes for check, lint, audit, test, docs, and ops validation.
- Apache-2.0 licensing metadata across workspace crates.

### Changed
- Harmonized governance contracts and generated registries for deterministic policy enforcement.

### Fixed
- Stabilized control-plane output normalization and deterministic artifact serialization.

### Breaking Changes
- none

### Included Surfaces
- `ops/`: stack, render, validation, k8s and load orchestration contracts.
- `configs/`: policy/config schema and inventory contracts.
- `makefiles/`: thin wrapper targets around canonical crate commands.

### Notes
- `v0.1.0` establishes the stable baseline for command, contract, and policy governance.
