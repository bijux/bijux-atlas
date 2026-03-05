---
title: Artifacts Locator
audience: contributor
type: reference
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
---

# Artifacts Locator

Use commands to locate artifacts instead of linking directly to raw artifact paths from reader docs.

## Real-data runs

1. `bijux-dev-atlas tutorials real-data list --format json`
2. `bijux-dev-atlas tutorials real-data run-all --dry-run --format json`
3. `bijux-dev-atlas tutorials real-data export-evidence --run-id <run_id> --format json`

## Docs generated reports

1. `bijux-dev-atlas docs where --format json`
2. `bijux-dev-atlas docs registry build --allow-write --format json`
3. `bijux-dev-atlas docs verify-generated --format json`

## Release and ops reports

1. `bijux-dev-atlas release ops compatibility-matrix --format json`
2. `bijux-dev-atlas release ops readiness-summary --format json`
3. `bijux-dev-atlas release validate --format json`
