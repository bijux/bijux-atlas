# Performance Contract

Required indexes:
- `idx_gene_summary_gene_id`
- `idx_gene_summary_name`
- `idx_gene_summary_name_normalized`
- `idx_gene_summary_biotype`
- `idx_gene_summary_region`
- `idx_gene_summary_cover_lookup`
- `idx_gene_summary_cover_region`
- `gene_summary_rtree`
- `idx_transcript_summary_transcript_id`
- `idx_transcript_summary_parent_gene_id`
- `idx_transcript_summary_biotype`
- `idx_transcript_summary_type`
- `idx_transcript_summary_region`

Hard gate:
- Dataset validation rejects artifacts if any required index above is missing.
- `atlas_meta.analyze_completed` must be `true` (ANALYZE required gate).
- Query plan regression gate runs `bijux dev atlas check run --suite ci_fast --json` in CI.

Query classes:
- `Cheap`: exact id lookups.
- `Medium`: exact name/biotype filters.
- `Heavy`: region and prefix queries.
- Transcript list endpoints are treated as heavy-class in server bulkheads.
- Region queries may fan out to shard DBs; shard selection is seqid-aware.

Max-work guard:
- Query cost estimator must remain bounded by `max_work_units`.
- Region requests are bounded by both span and estimated row count (`max_region_estimated_rows`).
- Full table scans are rejected unless explicitly allowed.

Search normalization:
- Name exact/prefix lookups use normalized `name_normalized` values.
- Normalization policy is Unicode `NFKC` + lowercase for deterministic collation behavior.

Bench policy:
- `benches/query_patterns.rs` supports baseline enforcement via `ATLAS_QUERY_BENCH_ENFORCE=1`.
- PR CI keeps benches non-blocking; nightly gates enforce regression thresholds.
