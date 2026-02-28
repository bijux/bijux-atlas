# Effects

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@ff4b8084`
- Reason to exist: define where side effects are allowed and where purity is required.

## Effect Policy

- Pure modules: planning and transformation code paths.
- Effectful modules: runtime wiring and storage adapters.
- API surfaces remain read-only and do not mutate dataset artifacts.

## Ports and Adapters Model

- Domain and planning code stays independent from transport and persistence details.
- Adapters isolate filesystem, network, and subprocess effects.
- Port contracts constrain adapter behavior and are validated by tests.

## Operational Relevance

Effect boundaries keep incident diagnostics explainable and prevent hidden runtime writes.

## What to Read Next

- [Architecture](index.md)
- [Boundaries](boundaries.md)
- [Storage](storage.md)

## Document Taxonomy

- Audience: `contributor`
- Type: `concept`
- Stability: `stable`
- Owner: `architecture`
