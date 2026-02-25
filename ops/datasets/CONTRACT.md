# Contract

- Area: `ops/datasets`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/datasets/manifest.json` | Authored dataset catalog SSOT |
| `ops/datasets/promotion-rules.json` | Authored promotion policy |
| `ops/datasets/qc-metadata.json` | Authored quality metadata |
| `ops/datasets/rollback-policy.json` | Authored rollback policy |
| `ops/datasets/real-datasets.json` | Authored real dataset registry |
| `ops/datasets/manifest.lock` | Authored lock and hashes |
| `ops/datasets/generated/dataset-index.json` | Generated dataset index |
| `ops/datasets/generated/dataset-lineage.json` | Generated lineage graph |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/datasets/manifest.json` | `ops/schema/datasets/manifest.schema.json` |
| `ops/datasets/manifest.lock` | `ops/schema/datasets/manifest-lock.schema.json` |
| `ops/datasets/promotion-rules.json` | `ops/schema/datasets/promotion-rules.schema.json` |
| `ops/datasets/qc-metadata.json` | `ops/schema/datasets/qc-metadata.schema.json` |
| `ops/datasets/rollback-policy.json` | `ops/schema/datasets/rollback-policy.schema.json` |
| `ops/datasets/generated/dataset-index.json` | `ops/schema/datasets/dataset-index.schema.json` |
| `ops/datasets/generated/dataset-lineage.json` | `ops/schema/datasets/dataset-lineage.schema.json` |

## Invariants

- Dataset promotion lifecycle is policy-driven: draft, promoted, and frozen states are enforced by rules and lock data.
- Rollback policy must map every promotable dataset namespace to a deterministic previous-good target.
- `manifest.lock` must contain stable hashes for any locked or mirrored dataset payload used by checks.
- Generated lineage and index artifacts must be deterministic for the same authored inputs.
- Fixtures under `ops/datasets/fixtures/` are versioned and immutable once referenced by tests.
- Dataset policy content in `ops/datasets` is authored truth; generated outputs must never be edited by hand.
