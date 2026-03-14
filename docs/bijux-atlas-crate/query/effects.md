# Effects Policy

`bijux-atlas-query` is a query planner/executor over explicit DB adapter input (`rusqlite::Connection`).

Allowed:
- Build SQL and execute against provided connection.
- Deterministic cursor/hash calculations.

Forbidden:
- HTTP/server framework dependencies.
- Direct runtime integration with API/server layers.
