# Load Test Metrics

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`

## Purpose

Define canonical metrics used by load suites and load reports.

## Core Metrics

- `http_req_duration`: request latency distribution.
- `http_req_failed`: failed request rate.
- `http_reqs`: total request throughput.
- `vus`: active virtual users.

## Resource and Saturation Signals

- process RSS and memory growth
- CPU saturation ratio
- disk I/O latency and throughput
- request queue depth
- thread pool utilization

## Suite Semantics

- `mixed-workload`: read/write mixed traffic.
- `ingest-query-workload`: ingest and query overlap behavior.
- `heavy-query-workload`: high-cost query path pressure.
- `read-heavy-workload`: read-dominant capacity behavior.
- `write-heavy-workload`: write-dominant pressure behavior.
- `long-running-stability`: sustained run with drift detection.
- `memory-leak-detection`: memory growth envelope verification.
- `cpu-saturation`: bounded degradation under CPU pressure.
- `disk-io-saturation`: bounded degradation under I/O pressure.
- `thread-pool-exhaustion`: queueing and timeout behavior under worker pressure.
