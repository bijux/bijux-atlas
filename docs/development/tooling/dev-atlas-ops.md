# Dev Atlas Ops Ownership

`bijux dev atlas ops` is the canonical control plane for ops validation, rendering, install planning, tool verification, and pins checks.

## Ownership Rules

- `ops/` contains SSOT inputs (inventory, stack manifests, pins, schemas, contracts).
- `makefiles/ops.mk` is delegation-only and must route to `bijux dev atlas ops ...`.
- CI workflows must call `make` wrappers or `bijux dev atlas ops ...`, not `bijux dev atlas ops ...`.
- Runtime artifacts are written only under `artifacts/atlas-dev/ops/...`.
- Governance control-plane ownership is Rust-only (`bijux-atlas-cli` + `bijux-dev-atlas`); Python runtime governance entrypoints are retired and must not be reintroduced.

## Canonical Entry Points

- `bijux dev atlas ops doctor`
- `bijux dev atlas ops validate`
- `bijux dev atlas ops pins check`
- `bijux dev atlas ops render --target kind --check`
- `bijux dev atlas ops verify-tools --allow-subprocess`
