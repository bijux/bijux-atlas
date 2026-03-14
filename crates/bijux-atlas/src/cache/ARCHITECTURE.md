# Server Cache Subsystem Architecture

## Responsibility

Own dataset cache lifecycle, connection limits, integrity checks, and breaker/quarantine behavior.

## Boundaries

- Reads immutable artifacts; does not mutate artifact SQLite.
- Exposes runtime cache state to server handlers/metrics.

## Effects

- FS for local cache materialization.
- Clock for TTL, breakers, and warmup timing.
