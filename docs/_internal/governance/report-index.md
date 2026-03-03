# Report Index

This page mirrors the governed report catalog from `configs/reports/reports.registry.json`.

Use `bijux-dev-atlas reports index --format human` to regenerate the markdown table below.

| Report ID | Version | Schema | Example |
| --- | --- | --- | --- |
| `closure-index` | `1` | `configs/contracts/reports/closure-index.schema.json` | `docs/_internal/generated/closure-index.json` |
| `docs-build-closure-summary` | `1` | `configs/contracts/reports/closure-summary.schema.json` | `artifacts/run/<run_id>/gates/docs/docs-build-closure-summary.json` |
| `docs-site-output` | `1` | `configs/contracts/reports/docs-site-output.schema.json` | `artifacts/run/<run_id>/gates/docs/site-output.json` |
| `helm-env` | `1` | `configs/contracts/reports/helm-env.schema.json` | `artifacts/contracts/ops/helm/helm-env-subset.json` |
| `ops-profiles` | `1` | `configs/contracts/reports/ops-profiles.schema.json` | `artifacts/contracts/ops/profiles/full-matrix.json` |
