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
- Schema registry: `configs/reports/schema-registry.json`
- Ownership registry: `configs/reports/ownership.json`
- Check to report mapping: `configs/reports/check-report-map.json`
- Runtime docs artifacts:
  - `docs/_internal/generated/closure-index.json`
  - `docs/_internal/generated/closure-index.md`
- Docs doctor run artifacts:
  - `artifacts/run/<run_id>/gates/docs/site-output.json`
  - `artifacts/run/<run_id>/gates/docs/docs-build-closure-summary.json`
  - `artifacts/run/<run_id>/gates/docs/report-manifest.json`

## Check to report mapping

| Check ID | Report file |
| --- | --- |
| `OPS-HELM-ENV-001` | `artifacts/contracts/ops/helm/helm-env-subset.json` |
| `OPS-PROFILES-001` | `artifacts/contracts/ops/profiles/full-matrix.json` |
| `OPS-PROFILES-002` | `artifacts/contracts/ops/profiles/full-matrix.json` |
| `OPS-PROFILES-003` | `artifacts/contracts/ops/profiles/full-matrix.json` |
| `OPS-PROFILES-004` | `artifacts/contracts/ops/profiles/full-matrix.json` |
| `OPS-DATASET-001` | `artifacts/contracts/ops/profiles/full-matrix.json` |
| `DOCS-SITE-001` | `artifacts/run/<run_id>/gates/docs/site-output.json` |
| `DOCS-SITE-002` | `artifacts/run/<run_id>/gates/docs/site-output.json` |
| `DOCS-SITE-003` | `artifacts/run/<run_id>/gates/docs/site-output.json` |
| `REPO-006` | `artifacts/contracts/repo/boundary-closure/summary.json` |

## How to verify

- `bijux dev atlas docs doctor --allow-subprocess --allow-write --format json`
- `bijux dev atlas docs site-dir --format json`
- `bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-DATASET-001`
- `bijux dev atlas artifacts report inventory --format json`
- `bijux dev atlas artifacts report read --report-path docs/_internal/generated/closure-index.json --format json`
- `bijux dev atlas artifacts report validate --reports-root docs/_internal/generated --format json`

## Consumption contract

- Read the report schema first, then read the report payload. The schema defines the stable envelope and required fields.
- Treat `status: "fail"` plus the paired `errors[]` list as the contract result, not stderr formatting.
- Use `report-manifest.json` as the index for a single docs doctor run instead of globbing ad hoc paths.
- Preserve the stable pretty JSON formatting when reviewing or copying report payloads so diffs stay meaningful.
- Prefer the registry files under `configs/reports/` over hand-maintained docs tables when automation needs ownership or check linkage.
- Use `artifacts report read` for a deterministic machine summary of one report and `artifacts report diff` for run-to-run drift.

## Failure triage guide

- `helm-env`: inspect `extra[]` first. Any key there means Helm emits a runtime env key the allowlist does not accept.
- `ops-profiles`: compare `helm_template`, `values_schema`, `dataset_validation`, and `kubeconform` in the same row to see which boundary failed.
- `docs-site-output`: check the missing path named in the failing rule before assuming the docs build itself is broken.
- `closure-summary`: use it as the roll-up entrypoint, then jump to the per-check report named in the summary row.

## Minimal reproduction

- Helm env subset:
  - `bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-HELM-ENV-001`
- Profile matrix and pinned datasets:
  - `bijux dev atlas ops profiles validate --allow-subprocess --profile-set rollout-safety --format json`
  - `bijux dev atlas contracts ops --mode effect --allow-subprocess --filter-contract OPS-DATASET-001`
- Docs site output:
  - `bijux dev atlas docs doctor --allow-subprocess --allow-write --format json`

## See also

- [Reference Index](../index.md)
- [Docs Site Output](../contracts/docs/site-output.md)
- [Report Schema Migrations](migrations/index.md)
