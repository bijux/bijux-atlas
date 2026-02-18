# Overload And Degradation Contract

- Owner: `bijux-atlas-server`
- Stability: `stable`

## What

Defines how request budgets, load shedding, and graceful degradation are enforced.

## Why

Protects cheap/critical reads during traffic spikes while bounding expensive work.

## Contracts

- Request budgets enforced before DB work:
  - `max page_size`
  - `max range span`
  - `max filter count`
  - `max response bytes`
- Request-size limits:
  - URI length cap
  - header-bytes cap
  - body bytes cap
- Rate limiting:
  - in-memory token bucket by default
  - stable `429` envelope and error code mapping
- Query-class bulkheads:
  - `cheap`, `medium`, `heavy` semaphores
  - per-class queue rejection when saturated
  - per-class timeout budgets via request/sql timeout policy
- Degradation policy:
  - cheap lookups remain available under pressure
  - non-cheap/heavy paths can be shed with stable retry semantics
- Metrics:
  - overload shedding active gauge
  - cheap-served-during-overload counter
  - request queue depth and class saturation signals

## Failure modes

- Without these limits, heavy traffic can starve cheap requests and cause cascading timeouts.

## How to verify

```bash
make dev-test-all
make ops-load-shedding
```
