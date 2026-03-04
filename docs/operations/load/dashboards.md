# Load Dashboards

- Owner: `bijux-atlas-operations`
- Type: `runbook`
- Audience: `operator`
- Stability: `stable`

## Purpose

Define dashboard panels for analyzing load suite behavior, regressions, and saturation.

## Required Panels

- `suite latency`: p50, p95, p99 per suite.
- `suite errors`: request failure rate and HTTP status class distribution.
- `suite throughput`: requests per second and active virtual users.
- `resource pressure`: CPU, memory, disk I/O, and queue depth.
- `degradation signals`: overload, shedding, timeout, and retry signals.

## Required Filters

- suite name
- run id
- dataset tier
- release revision
