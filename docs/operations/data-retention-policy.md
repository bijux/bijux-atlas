# Data Retention Policy

- Owner: `bijux-atlas-data`
- Type: `policy`
- Audience: `reviewers`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Last changed: `2026-03-03`
- Reason to exist: define what dataset-derived outputs are retained and why.

## Policy

- Keep governed dataset manifests and ingest reports for every release candidate.
- Keep fixture-derived ingest outputs long enough to reproduce the matching evidence bundle.
- Do not treat transient local scratch files as retained evidence; only recorded artifacts under `artifacts/` and `ops/release/` count.

## Verify

- The current evidence bundle lists the dataset manifest snapshot and ingest reports when they exist.
- The review packet references the retained dataset artifacts required for external review.
