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
python3 scripts/areas/layout/legacy_inventory.py --format text --json-out artifacts/evidence/legacy/inventory.json
python3 scripts/areas/layout/legacy_inventory.py --format text --check-policy --json-out artifacts/evidence/legacy/inventory.json
```

## Milestone

When `configs/policy/legacy-policy.json` sets `purge_enforced=true`, policy validation fails if any legacy entries remain.
