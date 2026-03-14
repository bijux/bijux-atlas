---
title: Real Data Reviewer Quick Path
audience: user
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Reviewer Quick Path

1. Open [Real Data Runs](/ops/tutorials/real-data/).
2. Confirm the generated table under [/_generated/real-data-runs-table/](/_generated/real-data-runs-table/).
3. Review evidence summary at [/_internal/generated/real-data-runs-overview/](/_internal/generated/real-data-runs-overview/).
4. Spot-check two run pages in `tiny` and `large-sample` tiers.
5. Validate one run locally with:
   - `bijux-dev-atlas tutorials real-data fetch --run-id <run-id> --format json`
   - `bijux-dev-atlas tutorials real-data ingest --run-id <run-id> --profile local --format json`
   - `bijux-dev-atlas tutorials real-data query-pack --run-id <run-id> --format json`
   - `bijux-dev-atlas tutorials real-data export-evidence --run-id <run-id> --profile local --format json`
