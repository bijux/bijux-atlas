# Evidence Sign-Off Workflow

- Owner: `bijux-atlas-operations`
- Purpose: `define how evidence bundles are reviewed and signed off before release decisions`
- Consumers: `checks_ops_human_workflow_maturity`
- Authority Tier: `explanatory`
- Audience: `reviewers`

## Workflow

1. Review evidence completeness checklist and evidence gap report.
2. Confirm readiness score and historical comparison are current and lineage-valid.
3. Confirm release evidence bundle paths and statuses are complete.
4. Record approvers and exceptions.
5. Attach sign-off outcome to release readiness decision.

## Required Inputs

- `ops/report/EVIDENCE_COMPLETENESS_CHECKLIST.md`
- `ops/_generated.example/evidence-gap-report.json`
- `ops/report/generated/readiness-score.json`
- `ops/report/generated/historical-comparison.json`
- `ops/report/generated/release-evidence-bundle.json`

## Required Sign-Off Roles

- Operations owner
- Observability owner
- Service/runtime owner

## Enforcement Links

- `checks_ops_evidence_bundle_discipline`
- `checks_ops_human_workflow_maturity`
