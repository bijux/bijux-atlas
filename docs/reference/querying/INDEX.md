# Querying Reference Index

- Owner: `bijux-atlas-query`

## What

Reference entrypoint for query semantics and budgets.

## Why

Centralizes pagination, ordering, filters, and cost behavior.

## Scope

Pagination, ordering, filters, cost estimator, sharding behavior.

## Non-goals

No deployment-specific tuning guidance.

## Contracts

- [Pagination](pagination.md)
- [Ordering](ordering.md)
- [Filters](filters.md)
- [Cost Estimator](cost-estimator.md)
- [Sharding Behavior](sharding.md)

## Failure modes

Ignoring query constraints can trigger policy rejection and overload safeguards.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-query
```

Expected output: planner, ordering, and cursor tests pass.


## See also

- [Performance Reference](../performance/INDEX.md)
- [Datasets Reference](../datasets/INDEX.md)
- [Security Reference](../security/INDEX.md)
