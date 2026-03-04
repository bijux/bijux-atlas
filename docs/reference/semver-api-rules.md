---
title: Semver API rules
audience: contributors
type: reference
stability: stable
owner: platform
last_reviewed: 2026-03-05
tags:
  - reference
  - release
  - semver
related:
  - configs/release/semver-api-policy.json
  - release/api-surface/golden
---

# Semver API rules

## Rule set

- Removed public API items are treated as breaking changes.
- Renamed public API items are treated as breaking changes (remove + add pattern).
- Added public API items are non-breaking unless they alter existing contracts.

## Enforcement surfaces

- `bijux-dev-atlas release api-surface snapshot`
- `bijux-dev-atlas release semver check`

## Machine policy

The machine-readable source is `configs/release/semver-api-policy.json`.
