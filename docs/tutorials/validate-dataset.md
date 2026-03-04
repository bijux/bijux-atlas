---
title: Tutorial: Validate Dataset
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - tutorial
  - validation
related:
  - docs/operations/workflows.md
  - docs/operations/validation-entrypoints.md
---

# Tutorial: Validate Dataset

## Goal

Validate dataset eligibility before promotion.

## Steps

1. Run validation entrypoints:

```bash
make contract-all
make check-all
```

2. Confirm no failures and no policy exceptions required.
3. Confirm evidence artifacts were produced for validation runs.

## Expected result

Dataset is marked promotion-ready with deterministic check outputs.
