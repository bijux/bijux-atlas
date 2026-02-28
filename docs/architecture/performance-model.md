# Performance model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: define caches, hot paths, bottlenecks, and overload behavior.

## Hot paths

- Query planning and execution for common API filters and pagination.
- Serving-store index lookups for release-bound dataset reads.
- API response assembly and transport serialization.

## Caches

- Runtime caches accelerate frequently accessed release metadata and query slices.
- Cache hits must preserve the same semantics as uncached paths.
- Cache bounds prevent untracked memory pressure.

## Bottlenecks

- Cold-start metadata warmup.
- Wide-range or low-selectivity query scans.
- I/O pressure on shared serving-store infrastructure.

## Overload and degradation

- Capacity controls prioritize critical read traffic.
- Degradation paths are explicit, measurable, and contract-checked.
- Observability stays available under degradation for operator triage.

## Latency targets

- p50 targets are optimized for common filtered query paths.
- p95 targets are maintained under sustained expected concurrency.
- p99 targets define overload thresholds and mitigation behavior.

## Limits and non-goals

- This model does not guarantee zero-latency behavior for unbounded scans.
- This model does not bypass correctness for raw throughput gains.

## Operational relevance

Operators need deterministic behavior when balancing stability against throughput.

## What this page is not

This page is not a load test runbook and not an SLO policy table.

## What to Read Next

- [Architecture](index.md)
- [Storage](storage.md)
- [Operations load](../operations/load/index.md)
- [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
