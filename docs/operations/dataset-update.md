# Dataset Update

- Owner: `bijux-atlas-data`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define the only supported way to add or replace governed dataset inputs.

Related ops contracts: `OPS-ROOT-023`, `DATA-001`.

## Prereqs

- Confirm the source files are available locally and do not require live network fetches during review.
- Confirm the dataset license is already allowed by `configs/datasets/manifest.yaml`.
- Confirm the replacement is compatible with the intended profile pin policy.

## Install

1. Add or update the dataset entry in `configs/datasets/manifest.yaml`.
2. Recompute and record the canonical checksum and size values.
3. Update `configs/datasets/pinned-policy.yaml` if the offline profile must pin the new dataset.
4. Run `cargo run -q -p bijux-dev-atlas -- datasets validate --format json`.
5. Run `cargo run -q -p bijux-dev-atlas -- ingest dry-run --dataset <dataset-id> --format json`.
6. Run `cargo run -q -p bijux-dev-atlas -- ingest run --dataset <dataset-id> --format json`.

## Verify

- `artifacts/datasets/datasets-manifest.json` reports `DATA-001` through `DATA-005` as true.
- `artifacts/ingest/ingest-plan.json` reports `INGEST-001=true`.
- `artifacts/ingest/ingest-run.json` reports `INGEST-002=true`.
- `artifacts/ingest/endtoend-ingest-query.json` reports `E2E-001=true` and `E2E-002=true`.

## Rollback

1. Restore the previous dataset row in `configs/datasets/manifest.yaml`.
2. Restore the previous pin set in `configs/datasets/pinned-policy.yaml` if it changed.
3. Re-run the same validation commands to confirm the older dataset set is still valid.
