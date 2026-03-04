# Ingest Reproducibility

- Owner: `platform`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`

## Purpose

This page defines the deterministic ingest contract for governed fixture datasets.

## Inputs

- The dataset must be declared in `configs/datasets/manifest.yaml`.
- The source fixture directory is part of the contract.
- Input checksums are captured in the dry-run plan.

## Outputs

- The dry-run plan declares the expected output paths.
- Output path names are stable and derived only from the dataset ID.
- The plan is written to `artifacts/ingest/ingest-plan.json`.

## Determinism Rules

- No network access is required.
- Input file checksums are the source of truth.
- The same dataset ID and source fixture produce the same declared output set.

## Verification

Run `bijux-dev-atlas ingest dry-run --dataset 110/homo_sapiens/GRCh38 --format json`.
