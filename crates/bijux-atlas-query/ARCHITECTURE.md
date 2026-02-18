# bijux-atlas-query Architecture

## Responsibility

Pure query planning/execution contract over SQLite-backed datasets.

## Boundaries

- No runtime server orchestration.
- No ingest pipeline logic.
- No async/network runtime dependencies (`tokio/reqwest/axum/hyper`).

## Effects

- Deterministic SQL building and cursor/filter semantics.
- SQLite query execution through passed connections.
