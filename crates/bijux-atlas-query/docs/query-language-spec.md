# Query Language Spec

## Scope

This crate supports a constrained, deterministic query subset for genes and transcripts.

## Gene Query Fields

Request structure (`GeneQueryRequest`):
- `filter.gene_id`
- `filter.name`
- `filter.name_prefix`
- `filter.biotype`
- `filter.region` (`seqid`, `start`, `end`)
- `limit`
- `cursor`

## Semantics

- Filters combine with logical `AND`.
- Cursor pagination is forward-only.
- Region filters use overlap semantics (`start <= end` and intersects row interval).
- Name lookup normalizes text before matching.

## Determinism Guarantees

- Parser output is typed (`GeneQueryAst`).
- Planner output is typed (`QueryPlan`) and normalized with stable predicate ordering.
- Sorting is stable (`seqid,start,gene_id` for region mode; `gene_id` otherwise).

## Examples

- Point lookup: `gene_id = ENSG000001`
- Prefix search: `name_prefix = BRCA`
- Region scan: `seqid=chr1, start=100, end=10000`

## Non-Goals

- Arbitrary SQL predicates
- Free-form joins
- Mutation operations
- Runtime-specific policy enforcement logic inside planner
