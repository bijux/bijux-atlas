# Ops Datasets

## Purpose
Own dataset manifest lock, pins, fetch/verify, QC, and promotion simulation.

## Entry points
- `make ops-datasets-verify`
- `make ops-dataset-qc`
- `make ops-dataset-promotion-sim`
- `make ops-publish-medium`

## Contracts
- `ops/datasets/CONTRACT.md`
- `ops/datasets/manifest.lock`
- `ops/datasets/promotion-rules.json`
- `ops/datasets/qc-metadata.json`
- `ops/datasets/fixture-policy.json`
- `ops/datasets/rollback-policy.json`

## Generated
- `ops/datasets/generated/dataset-index.json`
- `ops/datasets/generated/dataset-lineage.json`

## Artifacts
- `ops/_artifacts/<run_id>/datasets/`

## Failure modes
- Dataset lock/schema mismatch.
- QC threshold failure.
- Promotion simulation contract regression.
