# Promotion Record

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/datasets/promotion-rules.json`, `ops/datasets/rollback-policy.json`, `ops/_generated.example/fixture-drift-report.json`
- Contract references: `OPS-ROOT-023`, `OPS-DATASETS-003`, `OPS-DATASETS-006`, `OPS-DATASETS-007`

## What

Defines the canonical evidence and cross-links used when promoting fixture-backed dataset changes.

## Why

Promotion decisions must point to the same fixture drift and rollback policies that release reviewers use.

## Required Evidence

- Promotion rules: `ops/datasets/promotion-rules.json`
- Rollback policy: `ops/datasets/rollback-policy.json`
- Fixture drift report: `ops/_generated.example/fixture-drift-report.json`

## How To Verify

```bash
cargo test -q -p bijux-dev-atlas --test docs_ops_coherence_contracts -- --nocapture
```

Expected output: promotion docs stay linked to the canonical fixture governance evidence.
