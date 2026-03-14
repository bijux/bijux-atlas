# Ingest Performance Tuning Guide

## Tuning Sequence

1. Measure baseline with `ingest_scenarios` and `ingest_resource_tracking` benches.
2. Adjust `max_threads` and compare scaling benches.
3. Enable/disable sharding and compare shard generation and distribution benches.
4. Measure artifact compression impact before changing release artifact policies.

## Safe Tuning Rules

- Keep deterministic output invariants unchanged.
- Do not trade correctness checks for throughput gains.
- Validate anomaly and validation overhead after major changes.
