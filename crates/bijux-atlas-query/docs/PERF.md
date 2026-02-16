# Performance Contract

Required indexes:
- `idx_gene_summary_gene_id`
- `idx_gene_summary_name`
- `idx_gene_summary_biotype`
- `idx_gene_summary_region`
- `gene_summary_rtree`

Query classes:
- `Cheap`: exact id lookups.
- `Medium`: exact name/biotype filters.
- `Heavy`: region and prefix queries.

Max-work guard:
- Query cost estimator must remain bounded by `max_work_units`.
- Full table scans are rejected unless explicitly allowed.
