---
title: Tutorial: Filtering Queries
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - query
  - filtering
related:
  - docs/reference/querying/filtering.md
  - docs/configs/examples/datasets/specification.md
---

# Tutorial: Filtering Queries

## Goal

Filter records by field values and verify deterministic result ordering.

## Steps

1. Ensure runtime is started and dataset is ingested.
2. Run a query with filters for `chromosome` and `biotype`.
3. Compare returned records against expected constraints.

## Expected result

All rows satisfy filter predicates and ordering remains stable between runs.
