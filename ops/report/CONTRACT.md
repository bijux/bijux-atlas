# Contract

- Area: `ops/report`
- schema_version: `1`
- Canonical parent contract: `ops/CONTRACT.md`

## Authored vs Generated

| Path | Role |
| --- | --- |
| `ops/report/schema.json` | Authored unified report contract |
| `ops/report/evidence-levels.json` | Authored evidence policy |
| `ops/report/examples/unified-report-example.json` | Authored reference example |
| `ops/report/generated/report-diff.json` | Generated report diff |
| `ops/report/generated/historical-comparison.json` | Generated historical comparison |
| `ops/report/generated/release-evidence-bundle.json` | Generated evidence bundle |
| `ops/report/generated/readiness-score.json` | Generated readiness score |

## Schema References

| Artifact | Schema |
| --- | --- |
| `ops/report/schema.json` | `ops/schema/report/schema.json` |
| `ops/report/evidence-levels.json` | `ops/schema/report/evidence-levels.schema.json` |
| `ops/report/generated/report-diff.json` | `ops/schema/report/report-diff.schema.json` |
| `ops/report/generated/historical-comparison.json` | `ops/schema/report/historical-comparison.schema.json` |
| `ops/report/generated/release-evidence-bundle.json` | `ops/schema/report/release-evidence-bundle.schema.json` |
| `ops/report/generated/readiness-score.json` | `ops/schema/report/readiness-score.schema.json` |

## Invariants

- No duplicate authored truth is allowed; report model and evidence policy are authored only in `ops/report/schema.json` and `ops/report/evidence-levels.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/report`; report execution logic remains outside `ops/`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated report artifacts must include `generated_by` and `schema_version` metadata.
- Report docs must be linked from `docs/operations/ops-system/INDEX.md`; orphan docs are forbidden.
- Evidence bundle must include deterministic hashes of inventory and schema index inputs.
- Generated reporting outputs must be deterministic for identical authored and generated inputs.

## Enforcement Links

- `checks_ops_docs_governance`
- `checks_ops_evidence_bundle_discipline`
- `checks_ops_domain_contract_structure`
