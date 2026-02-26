# Contract

- Area: `ops/datasets`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `hybrid`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/datasets/manifest.json` | Authored dataset catalog SSOT |
| `ops/datasets/promotion-rules.json` | Authored promotion policy |
| `ops/datasets/consumer-list.json` | Authored dataset consumer contract |
| `ops/datasets/freeze-policy.json` | Authored dataset freeze and retention policy |
| `ops/datasets/qc-metadata.json` | Authored quality metadata |
| `ops/datasets/rollback-policy.json` | Authored rollback policy |
| `ops/datasets/real-datasets.json` | Authored real dataset registry |
| `ops/datasets/manifest.lock` | Authored dataset lock |
| `ops/datasets/generated/dataset-index.json` | Generated dataset index |
| `ops/datasets/generated/dataset-lineage.json` | Generated lineage graph |
| `ops/datasets/generated/fixture-inventory.json` | Generated fixture inventory |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/datasets/manifest.json` | `ops/schema/datasets/manifest.schema.json` |
| `ops/datasets/manifest.lock` | `ops/schema/datasets/manifest-lock.schema.json` |
| `ops/datasets/promotion-rules.json` | `ops/schema/datasets/promotion-rules.schema.json` |
| `ops/datasets/consumer-list.json` | `ops/schema/datasets/consumer-list.schema.json` |
| `ops/datasets/freeze-policy.json` | `ops/schema/datasets/freeze-policy.schema.json` |
| `ops/datasets/qc-metadata.json` | `ops/schema/datasets/qc-metadata.schema.json` |
| `ops/datasets/rollback-policy.json` | `ops/schema/datasets/rollback-policy.schema.json` |
| `ops/datasets/generated/dataset-index.json` | `ops/schema/datasets/dataset-index.schema.json` |
| `ops/datasets/generated/dataset-lineage.json` | `ops/schema/datasets/dataset-lineage.schema.json` |
| `ops/datasets/generated/fixture-inventory.json` | `ops/schema/datasets/fixture-inventory.schema.json` |

## Contract Taxonomy

- Structural contract: dataset manifest, lock, and policy artifacts define stable catalog and schema surfaces.
- Behavioral contract: promotion, rollback, and fixture lineage outputs define operational dataset behavior expectations.

## Invariants

- No duplicate authored truth is allowed; dataset identity and lifecycle policy are authored only under `ops/datasets/*.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/datasets`; execution logic lives outside `ops/`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated dataset artifacts must include `generated_by` and `schema_version` metadata.
- Dataset docs must be linked from `ops/datasets/INDEX.md`; orphan docs are forbidden.
- Fixture assets must be versioned under `ops/datasets/fixtures/**` and verified by `manifest.lock` hashes.
- Dataset promotion and rollback policy must be deterministic for a given manifest and lock set.
- Dataset consumer declarations must reference only dataset IDs present in `ops/datasets/manifest.json`.
- Dataset freeze policy must keep fixture assets append-only and enforce manifest-lock hash integrity.

## Runtime Evidence Mapping

- Fixture inventory evidence: `ops/datasets/generated/fixture-inventory.json`
- Lineage evidence: `ops/datasets/generated/dataset-lineage.json`
- Drift evidence: `ops/_generated.example/fixture-drift-report.json`

## Enforcement Links

- `checks_ops_domain_contract_structure`
- `checks_ops_required_files_contracts`
- `checks_ops_fixture_governance`
