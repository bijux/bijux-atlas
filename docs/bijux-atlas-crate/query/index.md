# Query Docs Index

Query surface invariants:
- Deterministic ordering per query mode.
- Cursor pagination with tamper protection.
- Strict limits and cost guards to prevent unbounded work.

Docs:
- [Architecture](architecture.md)
- [Public API](public-api.md)
- [Effects policy](effects.md)
- [Pagination contract](pagination.md)
- [Performance contract](perf.md)
- [Query performance](query-performance.md)
- [Query tuning guide](query-tuning-guide.md)
- [Query profiling guide](query-profiling-guide.md)
- [Query performance architecture](query-performance-architecture.md)
- [Query language spec](query-language-spec.md)
- [Ordering rules](ordering.md)
- [Cost estimator calibration](cost-estimator.md)
- [Adding filters safely](internal/adding-filters.md)
- [Transcript ordering](internal/transcript-ordering.md)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `public-api.md`, and add targeted tests.

- Central docs index: ../../index.md
- Crate README: ../../../crates/bijux-atlas/README.md
