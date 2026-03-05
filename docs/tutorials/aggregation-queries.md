---
title: Tutorial: Aggregation Queries
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - query
  - aggregation
related:
  - docs/reference/querying/index.md
  - docs/operations/query-performance-benchmarks.md
---

# Tutorial: Aggregation Queries

## Goal

Summarize dataset records by categorical dimensions.

## Steps

1. Query dataset with aggregation grouping by `chromosome`.
2. Validate count totals against source record count.
3. Repeat query and compare output for determinism.

## Expected result

Aggregated counts match source data and stay identical across repeated runs.
