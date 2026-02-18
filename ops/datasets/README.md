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

## Artifacts
- `ops/_artifacts/<run_id>/datasets/`

## Failure modes
- Dataset lock/schema mismatch.
- QC threshold failure.
- Promotion simulation contract regression.
