---
title: Tutorial: Debugging with Logs
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - logs
  - debugging
related:
  - docs/operations/observability/log-analysis-query-examples.md
  - docs/operations/runbooks/ingest-failures.md
---

# Tutorial: Debugging with Logs

## Goal

Use structured logs to isolate runtime and ingest issues quickly.

## Steps

1. Collect logs for failing time window.
2. Filter by request ID and severity.
3. Correlate log events with API errors.

## Expected result

Root cause path is identified with traceable log evidence.
