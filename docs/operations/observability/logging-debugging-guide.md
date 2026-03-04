# Logging Debugging Guide

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@053b86165`
- Reason to exist: provide deterministic steps for runtime debugging using structured logs.

## Workflow

1. Filter by `event_id` and `request_id`.
2. Correlate with `route` and `status`.
3. If query path, pivot by `query_id` and `dataset_id`.
4. Check retry and timeout events before escalating.
