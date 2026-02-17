# Query Docs Index

Query surface invariants:
- Deterministic ordering per query mode.
- Cursor pagination with tamper protection.
- Strict limits and cost guards to prevent unbounded work.

Docs:
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Effects policy](EFFECTS.md)
- [Pagination contract](PAGINATION.md)
- [Performance contract](PERF.md)
- [Ordering rules](ORDERING.md)
- [Cost estimator calibration](COST_ESTIMATOR.md)
- [Adding filters safely](ADDING_FILTERS.md)
- [Transcript ordering](TRANSCRIPT_ORDERING.md)
