---
title: Tutorial: Ingest Dataset
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - tutorial
  - ingest
related:
  - docs/operations/fixture-dataset-ingest.md
  - docs/operations/workflows.md
---

# Tutorial: Ingest Dataset

## Goal

Ingest a validated dataset and produce deterministic artifacts.

## Steps

1. Prepare source inputs and verify required files exist.
2. Run:

```bash
make ops-release-update
```

3. Verify readiness evidence:

```bash
make ops-readiness-scorecard
```

## Expected result

A new artifact set is generated with stable manifest and readiness checks passing.


## Tutorial dataset

Use `configs/examples/datasets/atlas-example-minimal` for reproducible results.
