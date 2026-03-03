# Ops Control Plane Contract

- Owner: `bijux-atlas-operations`
- Purpose: define the immutable control-plane boundary for ops automation.
- Consumers: `checks_ops_control_plane_doc_contract`, `checks_ops_ssot_manifests_schema_versions`
- Control plane version: `1`

## Scope

Ops control-plane behavior is driven by repository contracts and Rust control-plane commands.

## SSOT Rules

- `ops/SSOT.md` is the root SSOT index for ops domain records.
- Contract and inventory files under `ops/inventory/` define authoritative policy surfaces.

## Invariants

- Ops is specification-only.
- No runtime outputs are written under `ops/_generated.example/`.

## Effect Rules

- Human and CI entrypoints must route through `bijux dev atlas` wrappers.
- CI smoke runs `check run --suite ci` for deterministic coverage.
- Operator diagnostics include `bijux dev atlas doctor`.
