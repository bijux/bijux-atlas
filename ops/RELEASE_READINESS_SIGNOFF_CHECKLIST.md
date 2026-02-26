# Release Readiness Sign-Off Checklist

- Owner: `bijux-atlas-operations`
- Purpose: `define human sign-off requirements before a release readiness decision is accepted`
- Consumers: `checks_ops_human_workflow_maturity`

## Sign-Off Checklist

- [ ] `ops/report/generated/readiness-score.json` reviewed and reproducible
- [ ] `ops/report/generated/historical-comparison.json` reviewed for regressions
- [ ] `ops/report/generated/release-evidence-bundle.json` complete and lineage-valid
- [ ] SLO, alert, drill, and load mappings reviewed (`ops/inventory/scenario-slo-map.json`)
- [ ] Pin freeze and toolchain changes reviewed (`ops/inventory/pin-freeze.json`, `ops/inventory/toolchain.json`)
- [ ] Release blocking thresholds satisfied and exceptions documented
- [ ] Evidence gaps report status is `pass`

## Required Sign-Off Roles

- Operations owner
- Service/runtime owner
- Observability owner (when alerts, drills, or dashboards changed)

## Enforcement Links

- `checks_ops_evidence_bundle_discipline`
- `checks_ops_human_workflow_maturity`
