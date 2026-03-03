# Query model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@240605bb1dd034f0f58f07a313d49d280f81556c`
- Reason to exist: define query filters, sorting, cursors, and stability rules.

## Query semantics

- Filters and sorts are contract-defined and deterministic.
- Cursor pagination is stable for equivalent request parameters.
- Response ordering is explicit and not dependent on transient cache state.

## Stability rules

- Query semantics changes require explicit contract updates.
- Equivalent queries must yield equivalent ordered result sets.
- Degradation under overload remains explicit and observable.

## Terminology used here

- Cursor: [Glossary](../glossary.md)
- Dataset: [Glossary](../glossary.md)

## Next steps

- [Dataflow](dataflow.md)
- [API pagination](../api/pagination.md)
- [Reference querying](../reference/querying/index.md)
