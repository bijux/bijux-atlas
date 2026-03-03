# Review Packet

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@93ad533e5a4c4704f3a344db96b083570bb4d4b0`
- Reason to exist: list the exact evidence files to hand to an external reviewer.

## Required files

- `release/evidence/manifest.json`
- `release/evidence/identity.json`
- `release/evidence/bundle.tar`
- `configs/datasets/manifest.yaml`
- `artifacts/ops/<run_id>/reports/ops-simulation-summary.json`
- `artifacts/ops/<run_id>/reports/ops-lifecycle-summary.json`
- `artifacts/ops/<run_id>/reports/ops-drills-summary.json`
- `artifacts/ops/<run_id>/reports/ops-obs-verify.json`
- `artifacts/ingest/endtoend-ingest-query.json`

## Recommended attachments

- The matching `ops-drill-<name>.json` files used for the candidate review.
- The latest release candidate checklist and upgrade guide.
- The matching `artifacts/ingest/ingest-plan.json` and `artifacts/ingest/ingest-run.json` files.

## Auth model review

- Include `configs/security/auth-model.yaml` so reviewers can verify the declared trust boundary.
- Include `configs/security/policy.yaml` so reviewers can inspect the governed principal/action/resource model.

## Audit evidence

- Include `configs/observability/audit-log.schema.json`.
- Include `configs/observability/retention.yaml`.
- Include `artifacts/security/audit-verify.json`.
- Include `artifacts/security/log-field-inventory.json`.

## Exception inventory

- Include `configs/governance/exceptions.yaml`.
- Include `configs/governance/exceptions-archive.yaml`.
- Include `artifacts/governance/exceptions-summary.json`.
- Include `artifacts/governance/exceptions-expiry-warning.json`.

## Compatibility evidence

- Include `artifacts/governance/deprecations-summary.json`.
- Include `artifacts/governance/breaking-changes.json`.
- Include `artifacts/governance/governance-doctor.json`.
- Include `artifacts/governance/institutional-delta.md`.
