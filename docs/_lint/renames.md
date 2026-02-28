# Rename Ledger

- Owner: `docs-governance`
- Stability: `stable`

## What

Ledger of durable naming migrations applied to remove temporal/task naming drift.

## Why

Renames should stay explainable for future maintainers and reviewers.

## Entries

- `ops/load/k6/atlas_phase11.js` -> `ops/load/k6/mixed-80-20.js`
  - Reason: remove phase-based naming; describe traffic shape directly.
- `ops/observe/run/drill_*.sh` -> noun-led scenario names
  - Examples:
    - `drill_store_outage.sh` -> `store-outage.sh`
    - `drill_prom_outage.sh` -> `prom-outage.sh`
    - `drill_memory_growth.sh` -> `memory-growth.sh`
- `ops/e2e/k8s/tests/drill_pod_churn.sh` -> `ops/e2e/k8s/tests/pod-churn.sh`
  - Reason: scenario noun, no sequencing semantics.
- `docs/_drafts/canonical-transcript-policy-v2-stub.md` -> `docs/_drafts/transcripts-canonical-policy.md`
  - Reason: draft status instead of version-stub naming.
- `ops/load/experiments/` -> `ops/load/evaluations/`
  - Reason: durable term for non-gating analysis work.
- Config clarity renames:
  - `configs/perf/thresholds.json` -> `configs/perf/k6-thresholds.v1.json`
  - `configs/ops/cache-thresholds.json` -> `configs/ops/cache-budget-thresholds.v1.json`
  - `configs/ops/dataset-qc-thresholds.json` -> `configs/ops/dataset-qc-thresholds.v1.json`
- Snapshot naming clarity:
  - `ops/datasets/fixtures/medium/v1/golden_queries.json` -> `ops/datasets/fixtures/medium/v1/api-list-queries.v1.json`
  - `ops/datasets/fixtures/medium/v1/golden_snapshot.json` -> `ops/datasets/fixtures/medium/v1/api-list-responses.v1.json`
  - `crates/bijux-atlas-server/tests/golden/api_surface_response_snapshots.json` -> `crates/bijux-atlas-server/tests/snapshots/api-surface.responses.v1.json`
- Test intent renames:
  - `crates/bijux-atlas-server/tests/api_hardening.rs` -> `crates/bijux-atlas-server/tests/api-contracts.rs`
  - `crates/bijux-atlas-server/tests/latency_guard.rs` -> `crates/bijux-atlas-server/tests/p99-regression.rs`

## How To Verify

```bash
make rename-lint
```
