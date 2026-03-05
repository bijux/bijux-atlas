---
title: Tutorial: Query Dataset
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-04
tags:
  - tutorial
  - query
related:
  - docs/reference/querying/index.md
  - docs/reference/examples/query-command-example.md
---

# Tutorial: Query Dataset

## Goal

Run a query using the canonical query surface and verify response behavior.

## Steps

1. Ensure runtime is available locally or in target environment.
2. Execute a query command from the query example page.
3. Validate response shape against the documented contract.

Generated API surface excerpt:

--8<-- "_generated/openapi-snippets.md"

## Expected result

Stable response fields, deterministic ordering, and valid pagination metadata.


## Tutorial dataset

Use `configs/examples/datasets/atlas-example-minimal` for reproducible results.
