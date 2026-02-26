# Contract

- Area: `ops/report`
- schema_version: `1`
- contract_version: `1.0.0`
- contract_taxonomy: `hybrid`
- Canonical parent contract: `ops/CONTRACT.md`
- Owner: `bijux-atlas-operations`
- Purpose: `report contracts and generated operational summaries`
- Consumers: `checks_ops_docs_governance, checks_ops_domain_contract_structure`

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
| `ops/_generated.example/evidence-gap-report.json` | `ops/schema/report/evidence-gap-report.schema.json` |
| `ops/_generated.example/what-breaks-if-removed-report.json` | `ops/schema/report/what-breaks-if-removed-report.schema.json` |

## Contract Taxonomy

- Structural contract: report schema and evidence-level policy define stable report artifact structure.
- Behavioral contract: diff/comparison/readiness/evidence outputs define release behavior and governance decisions.

## Invariants

- No duplicate authored truth is allowed; report model and evidence policy are authored only in `ops/report/schema.json` and `ops/report/evidence-levels.json`.
- Schema references for this domain must resolve only to `ops/schema/**`.
- Behavior source is forbidden in `ops/report`; report execution logic remains outside `ops/`.
- The semantic domain name `obs` is prohibited; only canonical `observe` naming is valid.
- Generated report artifacts must include `generated_by` and `schema_version` metadata.
- Report docs must be linked from `docs/operations/ops-system/INDEX.md`; orphan docs are forbidden.
- Evidence bundle must include deterministic hashes of inventory and schema index inputs.
- Generated reporting outputs must be deterministic for identical authored and generated inputs.

## Runtime Evidence Mapping

- Report diff evidence: `ops/report/generated/report-diff.json`
- Historical evidence: `ops/report/generated/historical-comparison.json`
- Release evidence bundle: `ops/report/generated/release-evidence-bundle.json`
- Contract audit evidence: `ops/_generated.example/contract-audit-report.json`
- Contract dependency evidence: `ops/_generated.example/contract-dependency-graph.json`
- Deletion impact evidence: `ops/_generated.example/what-breaks-if-removed-report.json`

## Enforcement Links

- `checks_ops_docs_governance`
- `checks_ops_evidence_bundle_discipline`
- `checks_ops_domain_contract_structure`
