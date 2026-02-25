# Ops Reporting

- Owner: `bijux-atlas-operations`

Generic reporting surface for stack, observability, and load evidence.

## Scope

- Report generation and normalization for stack/obs/load evidence.
- Readiness scoring across contract outputs.
- Incident repro-kit bundle assembly.
- Historical comparison and diff outputs for release readiness.

## Contracts

- `ops/report/schema.json`
- `ops/report/evidence-levels.json`
- `ops/report/examples/unified-report-example.json`

## Generated

- `ops/report/generated/readiness-score.json`
- `ops/report/generated/report-diff.json`
- `ops/report/generated/historical-comparison.json`
- `ops/report/generated/release-evidence-bundle.json`

## Commands

- `make ops-report`
- `make ops-readiness-scorecard`
- `make ops-incident-repro-kit`

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
