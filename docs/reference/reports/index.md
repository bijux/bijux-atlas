# Reports Reference

- Owner: `docs-governance`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: provide one stable index for machine-readable report IDs and their meanings.

## Report schema rules

- Every governed report schema must declare `report_id`, `version`, and `inputs`.
- Governed reports intentionally omit timestamps so repeated runs stay deterministic.
- Reports must be rendered with stable pretty JSON so diffs remain readable.

## Canonical report IDs

- `helm-env`: rendered Helm env extraction report for the chart ConfigMap surface.
- `ops-profiles`: install profile matrix report covering Helm render, values schema, and kubeconform.
- `docs-site-output`: docs site output closure report for `mkdocs.yml` `site_dir`.
- `docs-build-closure-summary`: docs closure summary that combines link and site-output status.
- `closure-index`: generated boundary-closure index for human readers.
- `report-manifest`: stable manifest of reports emitted by a single docs doctor run.

## Where to find them

- Schemas: `configs/contracts/reports/*.schema.json`
- Runtime docs artifacts:
  - `docs/_internal/generated/closure-index.json`
  - `docs/_internal/generated/closure-index.md`
- Docs doctor run artifacts:
  - `artifacts/run/<run_id>/gates/docs/site-output.json`
  - `artifacts/run/<run_id>/gates/docs/docs-build-closure-summary.json`
  - `artifacts/run/<run_id>/gates/docs/report-manifest.json`

## How to verify

- `bijux dev atlas docs doctor --allow-subprocess --allow-write --format json`
- `bijux dev atlas docs site-dir --format json`

## See also

- [Reference Index](../index.md)
- [Docs Site Output](../contracts/docs/site-output.md)
