> Redirect Notice: canonical handbook content lives under `docs/operations/` (see `docs/operations/ops-system/INDEX.md`).

# Dataset Lifecycle Contract

Dataset lifecycle is governed by declarative contracts in `ops/datasets/`.

- Manifest source: `ops/datasets/manifest.json`
- Lock source: `ops/datasets/manifest.lock`
- Promotion rules: `ops/datasets/promotion-rules.json`
- QC metadata: `ops/datasets/qc-metadata.json`
- Fixture policy: `ops/datasets/fixture-policy.json`
- Rollback policy: `ops/datasets/rollback-policy.json`
- Generated index: `ops/datasets/generated/dataset-index.json`
- Generated lineage: `ops/datasets/generated/dataset-lineage.json`

Lifecycle guarantees:
- Dataset IDs are immutable once released.
- Promotion is append-only and pinned through `ops/inventory/pins.yaml`.
- QC and staleness checks are deterministic and contract-driven.
- Rollback switches dataset pointer without mutating old artifacts.
