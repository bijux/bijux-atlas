# Fixture Dataset Ingest

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/datasets/manifest.json`, `ops/datasets/generated/fixture-inventory.json`, `ops/datasets/fixture-policy.json`
- Contract references: `OPS-ROOT-023`, `OPS-DATASETS-001`, `OPS-DATASETS-002`, `OPS-DATASETS-007`

## What

Describes the canonical fixture dataset sources used for local ingest and regression checks.

## Why

Fixture ingest steps must point to the governed dataset manifests instead of ad-hoc local files.

## Canonical Inputs

- Dataset manifest: `ops/datasets/manifest.json`
- Fixture inventory: `ops/datasets/generated/fixture-inventory.json`
- Fixture policy: `ops/datasets/fixture-policy.json`

## How To Verify

```bash
cargo test -q -p bijux-dev-atlas --test docs_ops_coherence_contracts -- --nocapture
```

Expected output: docs references resolve to existing fixture manifests and generated inventories.
