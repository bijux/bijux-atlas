# Query model

- Owner: `architecture`
- Type: `concept`
- Audience: `contributor`
- Stability: `stable`
- Last verified against: `main@7dea4f4b9a65a61796b0f7ac8c2d185c0eaddb07`
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
