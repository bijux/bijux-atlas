# Performance Model

Owner: `architecture`  
Type: `concept`  
Reason to exist: define expected behavior under normal load and overload.

## Model

- Capacity protections prioritize critical read paths.
- Degradation is explicit and policy-controlled.
- Performance signals remain available during load shedding.

## Operational Relevance

Operators need deterministic behavior when balancing stability against throughput.

## Related Pages

- [Architecture](index.md)
- [Storage](storage.md)
