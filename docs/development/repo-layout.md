# Repository Layout

Owner: `platform`  
Type: `guide`  
Audience: `contributor`  
Reason to exist: describe canonical directory responsibilities and invariants.

## Invariants

- Product code lives in `crates/`.
- Operations assets live in `ops/`.
- Documentation lives in `docs/` and must use canonical section entrypoints.
- Root sprawl and legacy alias directories are not allowed.

## Repository Surface

- `crates/`: Rust workspace packages
- `configs/`: schema and contract data
- `ops/`: operational contracts and environment manifests
- `docs/`: canonical documentation source
- `makefiles/`: stable make wrappers
- `docker/`: container build and runtime contracts

## Build Entry Surface

Makefiles are thin wrappers over canonical crate and control-plane commands. Use `make` as a convenience layer, not as a hidden source of truth.
