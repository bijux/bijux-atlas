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

Hard gate:
- Dataset validation rejects artifacts if any required index above is missing.

Query classes:
- `Cheap`: exact id lookups.
- `Medium`: exact name/biotype filters.
- `Heavy`: region and prefix queries.

Max-work guard:
- Query cost estimator must remain bounded by `max_work_units`.
- Region requests are bounded by both span and estimated row count (`max_region_estimated_rows`).
- Full table scans are rejected unless explicitly allowed.
