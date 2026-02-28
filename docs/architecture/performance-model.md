# Performance Model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: define caches, hot paths, bottlenecks, and overload behavior.

## Hot Paths

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

## Overload and Degradation

- Capacity controls prioritize critical read traffic.
- Degradation paths are explicit, measurable, and contract-checked.
- Observability stays available under degradation for operator triage.

## Operational Relevance

Operators need deterministic behavior when balancing stability against throughput.

## What This Page Is Not

This page is not a load test runbook and not an SLO policy table.

## What to Read Next

- [Architecture](index.md)
- [Storage](storage.md)
- [Operations Load](../operations/load/index.md)
- [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
