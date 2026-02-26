# Ops Maturity Scorecard

- Owner: `bijux-atlas-operations`
- Purpose: `track maturity dimensions and evidence-backed status for ops governance`
- Consumers: `checks_ops_final_polish_contracts`

## Dimensions

| Dimension | Status | Evidence |
| --- | --- | --- |
| authority-governance | enforced | `ops/inventory/contracts-map.json`, `checks_ops_inventory_contract_integrity` |
| generated-lifecycle | enforced-partial | `ops/inventory/generated-committed-mirror.json`, `checks_ops_generated_lifecycle_metadata` |
| evidence-readiness | enforced-partial | `ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md`, `checks_ops_evidence_bundle_discipline` |
| portability | enforced-partial | `ops/PORTABILITY_MATRIX.md`, `checks_ops_portability_environment_contract` |
| deletion-safety | enforced-partial | `ops/MINIMAL_RELEASE_SURFACE.md`, `checks_ops_minimalism_and_deletion_safety` |
| human-workflow | enforced-partial | `ops/OPS_CHANGE_REVIEW_CHECKLIST.md`, `checks_ops_human_workflow_maturity` |

## Scoring Rules

- `enforced`: deterministic check coverage exists and is blocking
- `enforced-partial`: deterministic coverage exists, but runtime/CI execution proof is still external
- `documented`: contract exists but no blocking check

## Update Rule

- Update this scorecard whenever a new governance check closes or weakens a maturity gap.
