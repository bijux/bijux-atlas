# Unified Reporting Contract

Reporting outputs are contract-driven and deterministic.

Core contracts:
- `ops/report/schema.json`
- `ops/report/evidence-levels.json`
- `ops/report/examples/unified-report-example.json`

Generated artifacts:
- `ops/report/generated/readiness-score.json`
- `ops/report/generated/report-diff.json`
- `ops/report/generated/historical-comparison.json`
- `ops/report/generated/release-evidence-bundle.json`

Rules:
- report schema versions are pinned
- diff and historical outputs are machine-readable
- readiness score must be explicit and bounded [0,100]
- release bundle paths must resolve to existing artifacts
