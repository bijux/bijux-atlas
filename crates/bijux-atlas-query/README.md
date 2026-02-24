# bijux-atlas-query

Deterministic query parsing, planning, and execution for atlas gene/transcript read paths.

## Intended Use

- parse request payloads into typed AST
- produce pure typed plans with explicit cost hooks
- execute plans against SQLite-backed read models

## Supported Query Subset

- exact `gene_id`
- exact `name`
- `name_prefix`
- exact `biotype`
- region overlap (`seqid/start/end`)
- transcript filters (`parent_gene_id`, `biotype`, `transcript_type`, region)

## Determinism Guarantees

- stable AST normalization and formatting
- stable planner outputs for identical semantic requests
- stable ordering in query results and explain-plan normalization

## Docs

- `docs/QUERY_LANGUAGE_SPEC.md`
- `docs/ORDERING.md`
- `docs/PAGINATION.md`
- `docs/COST_ESTIMATOR.md`
