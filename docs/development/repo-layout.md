# Repository Layout

- Owner: `platform`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@d489531b`
- Reason to exist: describe canonical directory responsibilities and invariants.

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
- `make/`: stable make wrappers
- `docker/`: container build and runtime contracts
- Repository map (generated): `docs/reference/repo-map.md`

## Build Entry Surface

Makefiles are thin wrappers over canonical crate and control-plane commands.
Use `make` as a convenience layer, not as a hidden source of truth.

## Verify Success

Contributor changes should not introduce new top-level alias directories or duplicate entry surfaces.

## What to Read Next

- [Contributing](contributing.md)
- [Control-plane](../control-plane/index.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `guide`
- Stability: `stable`
- Owner: `platform`
