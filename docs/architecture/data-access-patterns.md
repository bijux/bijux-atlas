# Data Access Patterns

- Owner: `platform`
- Stability: `stable`

## Why

Define and prove the query shapes that must stay fast and index-backed.

## SSOT Inputs

- Critical queries: `configs/perf/critical_queries.json`
- Required SQLite indexes: `docs/contracts/SQLITE_INDEXES.json`
- Sort/tie-break contract: `docs/contracts/SORTS.json`

## Supported Query Shapes

1. Gene lookup by exact `gene_id` (cheap).
2. Gene list by `biotype` with stable pagination key (`gene_id`).
3. Region overlap query (`seqid` + `start/end`) using R*Tree + deterministic ordering.

## Stable Ordering Contract

- Non-region lists: `ORDER BY gene_id ASC`
- Region lists: `ORDER BY seqid ASC, start ASC, gene_id ASC`
- Tie-break key is always `gene_id`.

## Region Query Contract

- Region queries must use `gene_summary_rtree` and must not degrade to full table scan.
- R*Tree or interval index usage is required for release gates.

## How to Verify

```bash
make query-plan-gate
```

Expected: critical queries execute on fixture DB, EXPLAIN snapshots match, and no forbidden table scans are detected.
