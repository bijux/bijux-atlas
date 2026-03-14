# What belongs where

- Owner: `architecture`
- Type: `guide`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
- Reason to exist: give practical placement guidance for new code with explicit anti-patterns.

## Placement guide

- Domain invariants and shared primitives: `bijux-atlas-core`
- Shared domain types: `bijux-atlas-model`
- Ingest validation and artifact construction: `bijux-atlas-ingest`
- Serving-store access and persistence: `bijux-atlas-store`
- Policy parsing, evaluation, query semantics, and pagination behavior: `bijux-atlas`
- API surface behavior and transport: `bijux-atlas-api`
- Runtime process hosting: `bijux-atlas-server`
- Contributor checks and reports: `bijux-dev-atlas`

## Anti-patterns

- Adding runtime writes to API handlers.
- Introducing control-plane-only dependencies into runtime crates.
- Hiding cross-layer behavior behind helper modules without contract updates.

## Terminology used here

- Artifact: [Glossary](../glossary.md)
- Release: [Glossary](../glossary.md)

## Next steps

- [Crates map](crates-map.md)
- [Workspace boundaries rules](workspace-boundaries-rules.md)
- [How to add a new crate](how-to-add-a-new-crate.md)
