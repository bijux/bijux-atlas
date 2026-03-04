# Dataset Deprecation

- Owner: `bijux-atlas-data`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define how a governed dataset leaves active support without breaking reproducibility.

## Policy

- Do not delete a dataset entry until all profiles and ingest evidence no longer reference it.
- Mark the replacement dataset first, then migrate pinned policy consumers, then remove the deprecated row.
- Keep the historical checksum available long enough for old evidence bundles to remain explainable.

## Verify

- `datasets validate` still passes after the deprecation change.
- Existing release evidence keeps the old manifest snapshot that referenced the deprecated dataset.

## Rollback

- Restore the deprecated dataset row and previous pinned-policy values if any downstream consumer still depends on it.
