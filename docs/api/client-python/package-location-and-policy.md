---
title: Python Package Location And Policy
audience: contributor
type: policy
stability: stable
owner: api-contracts
last_reviewed: 2026-03-05
tags:
  - api
  - python
  - governance
---

# Python Package Location And Policy

`bijux-atlas` Python SDK content lives under `packages/bijux-atlas-python`.

Repository policy boundaries:

- Python source is allowed only under:
  - `packages/bijux-atlas-python/src/**/*.py`
  - `packages/bijux-atlas-python/tests/**/*.py`
  - `packages/bijux-atlas-python/examples/**/*.py`
- Notebooks are allowed only under:
  - `packages/bijux-atlas-python/notebooks/**/*.ipynb`
- `packages/bijux-atlas-python/control-plane/` is forbidden.
- `__pycache__` and `*.pyc` are forbidden.
- Root `clients/` is forbidden.

Verification entrypoints:

- `bijux-dev-atlas packages list`
- `bijux-dev-atlas packages verify`
- `bijux-dev-atlas checks automation-boundaries`
- `bijux-dev-atlas contract automation-boundaries`
