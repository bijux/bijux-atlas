# Cache Topology: Per-Node vs Per-Pod

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`
- Stability: `stable`

## Why

Local and CI environments use different storage pressure profiles. This page defines the tradeoff between per-node and per-pod cache topologies.

## Per-pod cache

- Isolated failure domain per pod.
- Simpler cleanup semantics.
- Higher duplicate bytes when multiple pods warm the same dataset.

## Per-node cache

- Better reuse across replicas on the same node.
- Lower cold-start latency after first pod warmup.
- Requires stricter permission and eviction policy controls.

## Contract

- Cache key is artifact hash, not dataset alias.
- Pinned datasets must never be evicted.
- Disk budget and hit-ratio thresholds are enforced by `make ops-cache-status`.
- Cache root must not be world-writable.

## Benchmarks

- Cold start: `make ops-perf-cold-start`
- Warm start: `make ops-perf-warm-start`
- Concurrent cold-start stampede: `cargo test -p bijux-atlas-server single_flight_download_shared_by_high_concurrency_calls -- --exact`

## Notes

- Use per-pod cache for predictable isolation in constrained dev machines.
- Use per-node cache for scale rehearsal and realistic warm-reuse patterns.

Related contracts: OPS-ROOT-023, OPS-ROOT-017.
