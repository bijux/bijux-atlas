# Query Connection Strategy

- Owner: `platform`
- Stability: `stable`

## Why

Keep query-serving behavior predictable under load and avoid write-side surprises.

## Per-request Strategy

- Each request acquires class bulkhead permits (`cheap`/`medium`/`heavy`) before query execution.
- Dataset SQLite handles are opened read-only via URI flags (`mode=ro&immutable=1`).
- Runtime applies `PRAGMA query_only=ON` and read-heavy tuning (`cache_size`, `mmap_size`).
- Prepared statements are primed and reused to reduce tail latency.

## Pool and Concurrency Policy

- Concurrency is bounded by per-class semaphores.
- Cheap traffic is isolated from heavy saturation.
- Dataset shard fanout uses separate shard permits.

## Busy Timeout Policy

- `busy_timeout=200ms` is set to absorb short lock contention.
- Long stalls are prevented by request/sql timeout budgets.

## Failure Behavior

- Startup/query path fails if SQLite cannot be opened read-only.
- Writes against dataset connections are rejected by SQLite (`readonly/query_only`).
- If bulkheads are saturated, non-cheap queries are shed first.
