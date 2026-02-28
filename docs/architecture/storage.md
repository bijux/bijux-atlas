# Storage

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: define serving-store schema, indexes, and migration philosophy.

## Serving-Store Model

- Atlas serves immutable release artifacts via a SQLite-backed serving store.
- Registry pointers resolve release IDs to immutable storage locations.
- Query surfaces read from serving-store structures without mutating published releases.

## Schema and Index Philosophy

- Schema changes are additive unless an explicit compatibility break is declared.
- Indexes are part of contract-governed performance behavior.
- Schema drift is blocked by contract and migration checks.

## Practical schema surfaces

| Surface | Purpose |
| --- | --- |
| release metadata tables | map release and alias state |
| serving indexes | accelerate filter/sort/cursor query paths |
| integrity metadata | preserve hash and compatibility checks |

## Migration Philosophy

- Forward-only migrations are preferred for deterministic release upgrades.
- Migration failures fail closed before runtime serving proceeds.
- Rollback strategy is release alias rollback, not ad-hoc schema edits.

## Limits and non-goals

- Storage model does not permit serving-path writes to immutable artifact source data.
- Storage model does not allow ad-hoc schema edits outside migration workflow.

## Caching Strategy

- Cache layers improve hot-path reads but never change correctness semantics.
- Cache policy is bounded and explicit to avoid untracked memory growth.

## Why SQLite Serving Store

SQLite provides deterministic local semantics, predictable deployment behavior, and strong compatibility control for immutable release reads.

See: [ADR 0002](../governance/adrs/adr-0002-sqlite-serving-store.md).

## Operational Relevance

Storage invariants define readiness behavior and prevent silent corruption under load.

## What This Page Is Not

This page is not a deployment recipe and not a SQL dump.

## What to Read Next

- [Architecture](index.md)
- [Dataflow](dataflow.md)
- [Performance Model](performance-model.md)
- [Store integrity model](store-integrity-model.md)
- [Glossary](../glossary.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
