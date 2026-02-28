# Crates philosophy

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define durable principles for crate boundaries and long-term maintainability.

- Keep crates small enough to preserve clear ownership and review scope.
- Keep dependency direction one-way across architecture layers.
- Keep contracts explicit at crate boundaries.
- Keep runtime and control-plane concerns separated.
- Keep effectful code isolated behind ports and adapters.

## Terminology used here

- Boundary: [Glossary](../glossary.md)
- Contract: [Glossary](../glossary.md)

## Next steps

- [Crates map](crates-map.md)
- [Workspace boundaries rules](workspace-boundaries-rules.md)
- [What belongs where](what-belongs-where.md)
