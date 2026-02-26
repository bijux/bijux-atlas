# Contract

- Area: `ops/inventory`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `structural`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/inventory/contracts-map.json` | Authored registry SSOT |
| `ops/inventory/authority-index.json` | Authored authority hierarchy index |
| `ops/inventory/pins.yaml` | Authored pins SSOT |
| `ops/inventory/pin-freeze.json` | Authored pin lifecycle policy |
| `ops/inventory/toolchain.json` | Authored toolchain registry |
| `ops/inventory/drill-contract-links.json` | Authored drill to contract linkage map |
| `ops/inventory/gates.json` | Authored gate catalog |
| `ops/inventory/surfaces.json` | Authored command surface catalog |
| `ops/inventory/layers.json` | Authored layering model |
| `ops/inventory/namespaces.json` | Authored namespace catalog |
| `ops/inventory/contracts.json` | Generated contracts mirror |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/inventory/contracts-map.json` | `ops/schema/inventory/contracts-map.schema.json` |
| `ops/inventory/authority-index.json` | `ops/schema/inventory/authority-index.schema.json` |
| `ops/inventory/contracts.json` | `ops/schema/inventory/contracts.schema.json` |
| `ops/inventory/pins.yaml` | `ops/schema/inventory/pins.schema.json` |
| `ops/inventory/pin-freeze.json` | `ops/schema/inventory/pin-freeze.schema.json` |
| `ops/inventory/drill-contract-links.json` | `ops/schema/inventory/drill-contract-links.schema.json` |
| `ops/inventory/toolchain.json` | `ops/schema/inventory/toolchain.schema.json` |
| `ops/inventory/gates.json` | `ops/schema/inventory/gates.schema.json` |
| `ops/inventory/surfaces.json` | `ops/schema/inventory/surfaces.schema.json` |
| `ops/inventory/layers.json` | `ops/schema/inventory/layers.schema.json` |
| `ops/inventory/namespaces.json` | `ops/schema/inventory/namespaces.schema.json` |

## Contract Taxonomy

- Structural contract: inventory artifacts define authoritative registry structure and metadata ownership.
- Behavioral contract: inventory gate/action mappings constrain execution surfaces through check ids and command ids.

## Invariants

- No duplicate authored truth is allowed; authoritative inventory registry is `ops/inventory/contracts-map.json`.
- `ops/inventory/contracts.json` is a generated output and must never become authored truth.
- Namespace identity lives in `ops/inventory/namespaces.json`; cross-domain dependency permissions live in `ops/inventory/layers.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/inventory`; inventory artifacts are declarative only.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated inventory mirror artifacts must include `generated_by` and `schema_version` metadata.
- Inventory docs must be linked from `ops/inventory/INDEX.md` or top-level `ops/INDEX.md`.
- Pin keys and values must follow canonical format and be deterministic under `pins.schema.json`.
- Registry outputs must be deterministic and stable for identical authored inventory inputs.
- Every drill id in `ops/inventory/drills.json` must map to at least one domain contract in `ops/inventory/drill-contract-links.json`.

## Ownership Semantics

- `OWNER.md` files identify accountable teams for review and drift response only.
- Effective machine ownership source is `ops/inventory/owners.json`.
- New ownership semantics must be added in `owners.json` first, then reflected in human-readable docs.

## Contract Fragment Lifecycle

- Domain fragments under `ops/inventory/contracts/*.contract.fragment.json` are input fragments only.
- Fragment changes must be reflected in the generated contracts mirror and pass contract-integrity checks in the same change set.
- Fragments without a runtime or check consumer are forbidden and must be removed.

## Cross-Domain Dependency Contract

- Inter-domain dependencies are declared only by `ops/inventory/layers.json` `layer_dependencies`.
- Cross-domain references outside declared dependency edges are invalid.
- New dependency edges must be reviewed with both producer and consumer owners.

## Runtime Evidence Mapping

- Registry drift evidence: `ops/_generated.example/registry-drift-report.json`
- Registry relationship evidence: `ops/_generated.example/registry-graph.json`
- Inventory checksum evidence: `ops/_generated.example/inventory-index.json`

## Enforcement Links

- `checks_ops_inventory_contract_integrity`
- `checks_ops_required_files_contracts`
- `checks_ops_domain_contract_structure`
