# Workspace boundaries rules

- Owner: `architecture`
- Type: `policy`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7f82f1b0`
- Reason to exist: define enforceable workspace dependency and ownership boundaries.

## Rules

- Runtime crates depend only on approved lower-layer crates.
- Control-plane crates orchestrate validation and reporting and do not implement runtime business logic.
- Ops and docs governance surfaces do not bypass runtime or contract boundaries.
- Cross-layer shortcuts require explicit architectural approval and contract updates.

## Verification

- Contract checks fail on undeclared cross-layer dependencies.
- Docs registry and ownership checks fail on boundary metadata drift.

## Terminology used here

- Boundary: [Glossary](../glossary.md)
- Lane: [Glossary](../glossary.md)

## Next steps

- [Boundaries](boundaries.md)
- [Crates map](crates-map.md)
- [How to add a new crate](how-to-add-a-new-crate.md)
