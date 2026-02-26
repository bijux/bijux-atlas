# Contract

- Area: `ops/env`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `structural`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/env/base/overlay.json` | Authored base overlay |
| `ops/env/dev/overlay.json` | Authored dev overlay |
| `ops/env/ci/overlay.json` | Authored CI overlay |
| `ops/env/prod/overlay.json` | Authored production overlay |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/env/base/overlay.json` | `ops/schema/env/overlay.schema.json` |
| `ops/env/dev/overlay.json` | `ops/schema/env/overlay.schema.json` |
| `ops/env/ci/overlay.json` | `ops/schema/env/overlay.schema.json` |
| `ops/env/prod/overlay.json` | `ops/schema/env/overlay.schema.json` |

## Contract Taxonomy

- Structural contract: environment overlays define stable configuration layers and merge surfaces.
- Behavioral contract: release and pin governance constrains overlay usage during runtime workflows.

## Invariants

- No duplicate authored truth is allowed; environment overlays are authored only in `ops/env/**/overlay.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/env`; overlays are declarative only.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Overlay JSON must include `schema_version` and `values` keys.
- Overlay docs must be linked from `ops/env/INDEX.md` or top-level `ops/INDEX.md`.
- Production overlay must remain deterministic and auditable for the same authored inputs.
- Environment contract changes must not bypass pin-freeze and release governance in inventory.

## Runtime Evidence Mapping

- Overlay schema evidence: `ops/schema/env/overlay.schema.json`
- Release evidence bundle: `ops/_generated.example/ops-evidence-bundle.json`
- Inventory index evidence: `ops/_generated.example/inventory-index.json`

## Enforcement Links

- `checks_ops_tree_contract`
- `checks_ops_required_files_contracts`
