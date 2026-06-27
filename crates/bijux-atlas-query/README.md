# bijux-atlas-query

`bijux-atlas-query` owns Atlas query behavior: request normalization, parsing,
planning, cursor encoding, SQLite execution, and the frozen query contract used
by downstream runtime surfaces.

Use this crate when you need:

- gene and transcript query request parsing
- deterministic query-plan classification and budgeting
- cursor encode or decode helpers for pagination
- SQLite-backed query execution and explain-plan inspection
- owned query benches and fixture contracts

It depends on `bijux-atlas-core` for canonical hashing and on
`bijux-atlas-model` for shared dataset, diff, and gene value types.
