# Ops Minimal Release Surface

- Owner: `bijux-atlas-operations`
- Purpose: `define the minimum ops files and generated evidence required to make a release readiness decision`
- Consumers: `checks_ops_minimalism_and_deletion_safety`

## Minimal Release Surface

- `ops/inventory/contracts-map.json`
- `ops/inventory/authority-index.json`
- `ops/inventory/registry.toml`
- `ops/load/suites/suites.json`
- `ops/observe/drills/drills.json`
- `ops/inventory/scenario-slo-map.json`
- `ops/inventory/drill-contract-links.json`
- `ops/report/generated/readiness-score.json`
- `ops/report/generated/historical-comparison.json`
- `ops/report/generated/release-evidence-bundle.json`
- `ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md`

## Deletion Impact Rules

- If a path in the minimal release surface is removed or renamed, the same commit must update all consuming checks, schemas, and contracts.
- Removal of a minimal release surface file requires an explicit replacement path and rationale in the same commit.
- Generated evidence files in the minimal release surface may be replaced only if lineage (`generated_by`, `generated_from`) remains enforced.

## Enforcement Links

- `checks_ops_minimalism_and_deletion_safety`
- `checks_ops_inventory_contract_integrity`
- `checks_ops_evidence_bundle_discipline`
