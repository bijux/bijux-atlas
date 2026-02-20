# Legacy Removal Plan

- Owner: `repo-surface`

## Scope

Short-lived tracker for removing legacy targets, scripts, and references.

## Current Policy

- Inventory source: `scripts/areas/layout/legacy_inventory.py`
- Evidence output: `artifacts/evidence/legacy/inventory.json`
- Baseline: `configs/policy/legacy-baseline.json`
- Policy: `configs/policy/legacy-policy.json`

## Commands

```bash
make legacy/list
make legacy/check
```

## Milestone

When `configs/policy/legacy-policy.json` sets `purge_enforced=true`, `make legacy/check` fails if any legacy entries remain.
