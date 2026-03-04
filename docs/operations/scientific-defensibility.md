# Scientific Defensibility

- Owner: `bijux-atlas-data`
- Type: `concept`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: tie dataset provenance and reproducible ingest to institutional review expectations.

## What Supports Defensibility

- A governed dataset manifest with explicit source, version, checksum, and license.
- Deterministic ingest planning and execution reports with stable input and output hashes.
- End-to-end verification that the stored artifacts and query path match the declared dataset.
- Evidence bundles that carry the dataset manifest snapshot and ingest reports alongside release artifacts.

## Verify

- Review `configs/datasets/manifest.yaml`, `artifacts/ingest/ingest-plan.json`, `artifacts/ingest/ingest-run.json`, and `artifacts/ingest/endtoend-ingest-query.json` together.
