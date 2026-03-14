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

`bijux-atlas` Python SDK content lives under `crates/bijux-atlas-python`.

Repository policy boundaries:

- Python source is allowed only under:
  - `crates/bijux-atlas-python/python/**/*.py`
  - `crates/bijux-atlas-python/tests/python/**/*.py`
  - `crates/bijux-atlas-python/examples/**/*.py`
- Notebooks are allowed only under:
  - `crates/bijux-atlas-python/notebooks/**/*.ipynb`
- `crates/bijux-atlas-python/tools/` is forbidden.
- `__pycache__` and `*.pyc` are forbidden.
- Root `clients/` is forbidden.

Verification entrypoints:

- `bijux-dev-atlas packages list`
- `bijux-dev-atlas packages verify`
- `bijux-dev-atlas checks automation-boundaries`
- `bijux-dev-atlas contract automation-boundaries`
