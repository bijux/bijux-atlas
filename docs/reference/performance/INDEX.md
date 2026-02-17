# Performance Reference Index

- Owner: `bijux-atlas-server`

## What

Reference entrypoint for latency and throughput controls.

## Why

Defines how p99 performance remains enforceable.

## Scope

Caching model, bulkheads, p99 strategy, perf regression policy, profiling.

## Non-goals

No benchmark marketing claims.

## Contracts

- [Caching Model](caching-model.md)
- [Bulkheads](bulkheads.md)
- [p99 Strategy](p99-strategy.md)
- [Perf Regression Policy](perf-regression-policy.md)
- [Profiling](profiling.md)

## Failure modes

Missing regression controls allows unnoticed latency drift.

## How to verify

```bash
$ make e2e-perf
```

Expected output: scenario scores satisfy SLO budgets.

## See also

- [Query Benchmarks](query-benchmarks.md)
- [k6 Ops](../../operations/load/k6.md)
- [SLO Targets](../../product/SLO_TARGETS.md)
