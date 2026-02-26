# Evidence Completeness Checklist

## Purpose

Defines the minimum release evidence set required for a readiness decision and the blocking conditions for evidence publication.

## Required Artifacts

- `ops/report/generated/readiness-score.json`
- `ops/report/generated/historical-comparison.json`
- `ops/report/generated/release-evidence-bundle.json`
- `ops/_generated.example/ops-evidence-bundle.json`
- `ops/_generated.example/scorecard.json`
- `ops/_generated.example/evidence-gap-report.json`

## Completeness Rules

- Every required artifact exists and is parseable.
- Every generated JSON evidence artifact includes `schema_version` and `generated_by`.
- Release evidence bundle `bundle_paths` includes readiness score and historical comparison artifacts.
- Evidence gap report status must be `pass` for release readiness.
- Historical comparison status must not be `regressed` when release bundle status is `ready`.

## Lineage Rules

- Release evidence bundle must declare `generated_by`.
- Release evidence bundle must declare `generated_from`.
- Evidence gap report must declare `generated_from` referencing the completeness checklist.

## Blocking Conditions

- Missing required evidence artifact.
- Missing lineage metadata.
- Evidence gap report status `fail`.
- Release evidence bundle status `ready` while historical comparison status is `regressed`.
