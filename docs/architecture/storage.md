# Storage

Owner: `architecture`  
Type: `concept`  
Reason to exist: define serving-store and cache invariants.

## Invariants

- Published artifacts are immutable.
- Integrity mismatches block dataset open.
- Cache behavior is bounded by policy-driven limits.

## Operational Relevance

Storage invariants define readiness behavior and prevent silent corruption under load.

## Related Pages

- [Architecture](index.md)
- [Dataflow](dataflow.md)
