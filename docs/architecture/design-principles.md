# Design Principles

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: capture the durable design principles that guide architecture decisions.

## Core Principles

- Determinism over convenience in release-critical behavior.
- Explicit boundaries over implicit cross-layer shortcuts.
- Contracts before assumptions for runtime and API surfaces.
- Observability by default for failure diagnosis.
- Stable reader-facing docs with contributor-only enforcement machinery separated.

## Architectural Application

- Ingest stages are deterministic and schema-governed.
- Query behavior is stable and compatibility-aware.
- Ops and release processes require evidence, not narrative-only approval.
- Governance references remain linked to executable checks and contracts.

## Terminology used here

- Determinism: [Glossary](../glossary.md)
- Contract: [Glossary](../glossary.md)

## Next steps

- [Dataflow](dataflow.md)
- [Boundaries](boundaries.md)
- [Crates philosophy](crates-philosophy.md)
