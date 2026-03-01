# PUBLIC API: bijux-atlas-query

Stability reference: [Stability Levels](../../../docs/_internal/style/stability-levels.md)

Stable exports:
- `CRATE_NAME`
- Query models: `GeneFields`, `GeneFilter`, `RegionFilter`, `GeneQueryRequest`, `GeneRow`, `GeneQueryResponse`
- Limits: `QueryLimits`
- Planner: `QueryClass`, `classify_query`, `estimate_work_units`
- Execution: `query_genes`, `explain_query_plan`
- Projection helpers: `compile_field_projection`, `escape_like_prefix`
- Normalization: `query_normalization_hash`
- Cursor errors: `CursorError`, `CursorErrorCode`
- Query errors: `QueryError`, `QueryErrorCode`
