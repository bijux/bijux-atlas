# Data Retention Policy

- Owner: `bijux-atlas-data`
- Type: `policy`
- Audience: `institutional-reader`
- Stability: `stable`
- Last verified against: `main@522431fd5e6376d1fdc88f630ae5d334612ca8d2`
- Last changed: `2026-03-03`
- Reason to exist: define what dataset-derived outputs are retained and why.

## Policy

- Keep governed dataset manifests and ingest reports for every release candidate.
- Keep fixture-derived ingest outputs long enough to reproduce the matching evidence bundle.
- Do not treat transient local scratch files as retained evidence; only recorded artifacts under `artifacts/` and `release/` count.

## Verify

- The current evidence bundle lists the dataset manifest snapshot and ingest reports when they exist.
- The review packet references the retained dataset artifacts required for external review.
