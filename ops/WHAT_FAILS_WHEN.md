# What Fails When

- Owner: `bijux-atlas-operations`
- Purpose: `document failure impact mapping from key ops components to checks, evidence, and release outcomes`
- Consumers: `checks_ops_final_polish_contracts`
- Authority Tier: `explanatory`
- Audience: `operators`

## Failure Impact Mapping

| Component | Immediate Failure | Detected By | Release Impact |
| --- | --- | --- | --- |
| `ops/inventory/contracts-map.json` | authority graph drift, orphan truth | `checks_ops_inventory_contract_integrity` | block |
| `ops/schema/generated/compatibility-lock.json` | schema compatibility proof unavailable | `checks_ops_schema_presence` | block |
| `ops/report/generated/readiness-score.json` | readiness decision unsupported | `checks_ops_evidence_bundle_discipline` | block |
| `ops/report/generated/historical-comparison.json` | regression comparison unavailable | `checks_ops_evidence_bundle_discipline` | block |
| `ops/observe/drills/drills.json` | drill linkage and observability proof gaps | `checks_ops_fixture_governance`, `checks_ops_human_workflow_maturity` | block |
| `ops/load/suites/suites.json` | load evidence and SLO mapping drift | `checks_ops_fixture_governance`, `checks_ops_minimalism_and_deletion_safety` | block |

## Deletion Impact Rule

- If a path in this table changes, update the table, the consuming checks, and any affected sign-off/evidence contracts in the same commit series.
