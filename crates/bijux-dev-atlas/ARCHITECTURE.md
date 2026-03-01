# bijux-dev-atlas Architecture

This crate is the control-plane executable for repository governance.
It orchestrates checks and reports while keeping execution deterministic.
All transient build outputs are written under `artifacts/target`.

## Execution policies

- Benchmark groups and output names remain unique.
- Contracts run through stable registries and deterministic ordering.
- Artifact write paths are explicit and reviewed.
- Runtime side effects are guarded by capability flags.

## Documentation boundaries

The public contract and operational guidance live under `docs/`.
This file defines only crate-internal architecture boundaries.
