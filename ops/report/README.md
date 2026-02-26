# Ops Reporting

- Owner: `bijux-atlas-operations`
- Purpose: `report contracts and generated operational summaries`
- Consumers: `bijux dev atlas ops report, checks_ops_domain_contract_structure`

Report assembly and evidence publishing across stack, observe, load, datasets, and e2e.

## Start Here

- `ops/report/schema.json`
- `ops/report/evidence-levels.json`
- `ops/report/examples/unified-report-example.json`
- `docs/operations/ops-system/INDEX.md`

## Generated

- `ops/report/generated/readiness-score.json`
- `ops/report/generated/report-diff.json`
- `ops/report/generated/historical-comparison.json`
- `ops/report/generated/release-evidence-bundle.json`

## Entrypoints

- `make ops-report`
- `make ops-readiness-scorecard`
- `make ops-incident-repro-kit`
