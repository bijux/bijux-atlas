# Runtime Model

The server runtime is async-first and executes request paths on Tokio.

## Boundaries

- Async request execution: HTTP handlers, cache refresh, Redis, and store/network access.
- Blocking work is allowed only behind explicit adapters for filesystem and SQLite interaction.
- Background catalog refresh and metrics collection run on async tasks.

## External I/O Policy

- HTTP clients use async `reqwest::Client`.
- Every external request must define explicit timeouts.
- External calls are traced with spans so latency and retry behavior are observable.

## Blocking Policy

- `reqwest` blocking mode is forbidden in `bijux-atlas-server`.
- `reqwest::blocking` imports are forbidden in server sources.
- If blocking network I/O is required, it belongs in non-server tools (CLI/store utilities), never in request handlers.
