# Logging Debugging Guide

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: provide deterministic steps for runtime debugging using structured logs.

## Workflow

1. Filter by `event_id` and `request_id`.
2. Correlate with `route` and `status`.
3. If query path, pivot by `query_id` and `dataset_id`.
4. Check retry and timeout events before escalating.
