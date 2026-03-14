# bijux-atlas-server Architecture

## Responsibility

HTTP/runtime orchestration only: transport, cache integration, store integration, and observability.

## Boundaries

- Depends on API/model/query/store contracts.
- Must not depend on ingest internals.
- Artifact SQLite access is read-only.

## Effects

- IO: SQLite reads, cache filesystem reads/writes.
- Net: HTTP serving and upstream store access via adapters.
- FS: cache files only; no artifact mutation.
- Clock: timeouts, rate limits, metrics timings.
