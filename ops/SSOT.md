# Ops SSOT

- Owner: `bijux-atlas-operations`
- Purpose: define operational SSOT boundaries and canonical inventories.
- Consumers: `checks_ops_ssot_manifests_schema_versions`

## Boundary

`ops/` is specification-only for operational contracts, inventories, runbooks, and evidence models.
Narrative governance content is canonical under `docs/governance/` and referenced from `ops/` via stubs.

## Canonical Manifests

- `ops/inventory/contracts-map.json`
- `ops/inventory/authority-index.json`
- `ops/inventory/surfaces.json`
- `ops/inventory/toolchain.json`
- `ops/inventory/generated-committed-mirror.json`

## Generated Evidence Model

- Runtime-generated outputs: `ops/_generated/`
- Committed curated examples: `ops/_generated.example/`
