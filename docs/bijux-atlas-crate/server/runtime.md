# Runtime

## Architecture

`bijux-atlas-server` runtime is intentionally split into three layers:

- `http/`: request/response mapping only.
- `runtime/`: orchestration and state transitions.
- `effect_adapters/`: direct I/O adapters (filesystem, store, clock boundaries).

This keeps business flow deterministic and testable while confining side effects.

## Invariants

- HTTP handlers do not perform direct filesystem or network side effects.
- Runtime orchestration owns bulkheads, timeouts, and circuit-breaker decisions.
- Effect adapters are the only place allowed to call external systems directly.
- Dataset open/publish flows use atomic staging and verification before promotion.

## Shutdown

- SIGTERM triggers graceful drain.
- Heavy query permits are drained first.
- In-flight requests are bounded by configured deadlines.
- New work admission is reduced while shutdown is in progress.

## Cached-Only Mode

- When enabled, readiness can remain healthy while remote store is unavailable.
- Requests for uncached datasets fail fast with stable service-unavailable errors.
