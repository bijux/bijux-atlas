# Telemetry model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define required metrics and spans by subsystem boundaries.

## Required telemetry surfaces

- Ingest: validation counts, failure categories, artifact publication latency.
- Store: read latency, cache hit ratio, database error count.
- Query: request rate, p50/p95/p99 latency, result-size distribution.
- API: request status distribution, dependency failures, timeout counts.
- Control-plane: check runtime, gate outcomes, evidence generation timing.

## Span model

- One root span per user-visible operation.
- Child spans at ingest, store, query, and API boundaries.
- Error spans include category, code, and retry hint when applicable.

## Terminology used here

- Lane: [Glossary](../glossary.md)
- Fixture: [Glossary](../glossary.md)

## Next steps

- [Performance model](performance-model.md)
- [Operations observability](../operations/observability/index.md)
- [Reference contracts telemetry](../reference/contracts/telemetry.md)
