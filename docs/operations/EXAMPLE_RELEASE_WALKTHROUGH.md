# Example Release Walkthrough

- Owner: `bijux-atlas-operations`
- Purpose: `show a representative release readiness decision flow using evidence and sign-off contracts`
- Consumers: `checks_ops_final_polish_contracts`

## Steps

1. Review `ops/report/generated/readiness-score.json`.
2. Review `ops/report/generated/historical-comparison.json` for regressions.
3. Validate `ops/report/generated/release-evidence-bundle.json` completeness and lineage.
4. Check sign-off readiness with `ops/RELEASE_READINESS_SIGNOFF_CHECKLIST.md`.
5. Confirm pin/toolchain changes and schema compatibility impacts.
6. Record release decision and evidence references.

## Linked Contracts

- `ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md`
- `ops/RELEASE_READINESS_SIGNOFF_CHECKLIST.md`
- `ops/SCHEMA_EVOLUTION_WORKFLOW.md`
