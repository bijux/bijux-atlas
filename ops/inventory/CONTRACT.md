# Contract

- Area: `ops/inventory`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/inventory/contracts-map.json` | Authored registry SSOT |
| `ops/inventory/pins.yaml` | Authored pins SSOT |
| `ops/inventory/pin-freeze.json` | Authored pin lifecycle policy |
| `ops/inventory/toolchain.json` | Authored toolchain registry |
| `ops/inventory/gates.json` | Authored gate catalog |
| `ops/inventory/surfaces.json` | Authored command surface catalog |
| `ops/inventory/layers.json` | Authored layering model |
| `ops/inventory/namespaces.json` | Authored namespace catalog |
| `ops/inventory/contracts.json` | Generated contracts mirror |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/inventory/contracts-map.json` | `ops/schema/inventory/contracts-map.schema.json` |
| `ops/inventory/contracts.json` | `ops/schema/inventory/contracts.schema.json` |
| `ops/inventory/pins.yaml` | `ops/schema/inventory/pins.schema.json` |
| `ops/inventory/pin-freeze.json` | `ops/schema/inventory/pin-freeze.schema.json` |
| `ops/inventory/toolchain.json` | `ops/schema/inventory/toolchain.schema.json` |
| `ops/inventory/gates.json` | `ops/schema/inventory/gates.schema.json` |
| `ops/inventory/surfaces.json` | `ops/schema/inventory/surfaces.schema.json` |
| `ops/inventory/layers.json` | `ops/schema/inventory/layers.schema.json` |
| `ops/inventory/namespaces.json` | `ops/schema/inventory/namespaces.schema.json` |

## Invariants

- No duplicate authored truth is allowed; authoritative inventory registry is `ops/inventory/contracts-map.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/inventory`; inventory artifacts are declarative only.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated inventory mirror artifacts must include `generated_by` and `schema_version` metadata.
- Inventory docs must be linked from `ops/inventory/INDEX.md` or top-level `ops/INDEX.md`.
- Pin keys and values must follow canonical format and be deterministic under `pins.schema.json`.
- Registry outputs must be deterministic and stable for identical authored inventory inputs.

## Enforcement Links

- `checks_ops_inventory_contract_integrity`
- `checks_ops_required_files_contracts`
- `checks_ops_domain_contract_structure`
