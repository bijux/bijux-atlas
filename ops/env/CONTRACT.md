# Contract

- Area: `ops/env`
- schema_version: `1`
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

## Invariants

- No duplicate authored truth is allowed; environment overlays are authored only in `ops/env/**/overlay.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/env`; overlays are declarative only.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Overlay JSON must include `schema_version` and `values` keys.
- Overlay docs must be linked from `ops/env/INDEX.md` or top-level `ops/INDEX.md`.
- Production overlay must remain deterministic and auditable for the same authored inputs.
- Environment contract changes must not bypass pin-freeze and release governance in inventory.

## Enforcement Links

- `checks_ops_tree_contract`
- `checks_ops_required_files_contracts`
