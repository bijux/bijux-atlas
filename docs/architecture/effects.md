# Effects

Owner: `architecture`  
Type: `concept`  
Reason to exist: define where side effects are allowed and where purity is required.

## Effect Policy

- Pure modules: planning and transformation code paths.
- Effectful modules: runtime wiring and storage adapters.
- API surfaces remain read-only and do not mutate dataset artifacts.

## Operational Relevance

Effect boundaries keep incident diagnostics explainable and prevent hidden runtime writes.

## Related Pages

- [Architecture](index.md)
- [Boundaries](boundaries.md)
- [Storage](storage.md)
