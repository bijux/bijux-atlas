# Patterns

- Adapter pattern: parse wire params into validated API DTOs; server maps DTOs into query-layer structs.
- Error translation boundary: wire-layer codes remain stable even if backend internals evolve.
- Projection contract: `include=` validated in API before query execution.
