# Real Dataset Catalog

This directory is the source of truth for the ten real-data tutorial datasets used by `bijux-dev-atlas`.

Each dataset directory contains:

- `fetch-spec.json`: canonical source URL and expected SHA-256 digest.
- `metadata.json`: minimal dataset identity metadata (`dataset_id`, `expected_sha256`, `description`).
- `dataset-contract.json`: ingest/query expectations used by contracts.
- `query-pack.json` and `queries.sql`: deterministic query set definitions.
- `ingest-map.json` and `normalization-rules.json`: ingest transformation policy.
- `golden-summary-metrics.json`: summary expectations for regression checks.

Downloads are stored outside source control under:

- `artifacts/tutorials/datasets/<dataset>/` for fetched payloads and provenance.
- `artifacts/tutorials/runs/<run_id>/` for ingest/query/evidence outputs.

Run one dataset end-to-end:

```bash
cargo run -p bijux-dev-atlas -- tutorials run dataset-e2e --dataset-id genes-baseline --profile local --format json
```

Offline replay from cached dataset (no network):

```bash
cargo run -p bijux-dev-atlas -- tutorials run dataset-e2e --dataset-id genes-baseline --profile local --no-fetch --format json
```
