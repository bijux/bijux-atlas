# Dataset Deprecation

- Owner: `bijux-atlas-data`
- Type: `policy`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@522431fd5e6376d1fdc88f630ae5d334612ca8d2`
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
