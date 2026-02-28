# Design principles

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: capture the durable design principles that guide architecture decisions.

- Determinism over convenience in release-critical behavior.
- Explicit boundaries over implicit cross-layer shortcuts.
- Contracts before assumptions for runtime and API surfaces.
- Observability by default for failure diagnosis.
- Stable reader-facing docs with contributor-only enforcement machinery separated.

## Terminology used here

- Determinism: [Glossary](../glossary.md)
- Contract: [Glossary](../glossary.md)

## Next steps

- [Dataflow](dataflow.md)
- [Boundaries](boundaries.md)
- [Crates philosophy](crates-philosophy.md)
