# Query Profiling Guide

- Owner: `bijux-atlas-query`
- Stability: `stable`

## Purpose

Provide a repeatable profiling workflow for query planner, executor, serialization, and cache behavior.

## Profiling Workflow

1. Build benches: `cargo bench -p bijux-atlas-query --no-run`.
2. Run focused bench: `cargo bench -p bijux-atlas-query query_planner_and_serialization`.
3. Run cache bench: `cargo bench -p bijux-atlas-query query_cache`.
4. Run routing/index bench: `cargo bench -p bijux-atlas-query query_routing_and_index`.
5. Compare output with baseline fixtures and report deltas.

## What to Capture

- Scenario name and dataset tier
- p50/p95/p99 latency
- Throughput and concurrency level
- Cache hit and miss rates
- Explain-plan output for indexed scenarios
