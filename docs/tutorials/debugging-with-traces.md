---
title: Tutorial: Debugging with Traces
audience: user
type: guide
stability: stable
owner: docs-governance
last_reviewed: 2026-03-05
tags:
  - tutorial
  - traces
  - debugging
related:
  - docs/operations/observability/trace-dashboard-examples.md
  - docs/operations/observability/tracing-dashboard-examples.md
---

# Tutorial: Debugging with Traces

## Goal

Use distributed trace spans to diagnose latency and failure points.

## Steps

1. Capture trace for slow or failed request.
2. Inspect span timeline and error attributes.
3. Confirm bottleneck location and dependent service impact.

## Expected result

Trace data identifies the specific component causing the issue.
