---
title: Tutorial: Streaming Queries
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - query
  - streaming
related:
  - docs/reference/querying/cursor-usage.md
  - docs/api/client-python-quickstart.md
---

# Tutorial: Streaming Queries

## Goal

Consume paginated query results as a streaming iterator.

## Steps

1. Start with a query using a bounded page size.
2. Iterate through `next_page_token` until no token remains.
3. Track total rows emitted and compare with expected count.

## Expected result

Rows stream without duplication and final row count is complete.
