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
- [Query language spec](query-language-spec.md)
- [Ordering rules](ordering.md)
- [Cost estimator calibration](cost-estimator.md)
- [Adding filters safely](adding-filters.md)
- [Transcript ordering](transcript-ordering.md)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
