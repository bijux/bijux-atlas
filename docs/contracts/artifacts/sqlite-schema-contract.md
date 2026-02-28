# SQLite Schema Contract (Ingest)

- Owner: `bijux-atlas-ingest`
- Stability: `stable`

## SSOT

- Canonical schema source: `crates/bijux-atlas-ingest/sql/schema_v4.sql`
- Embedded at build-time via `include_str!` in ingest SQLite writer.
- Guarded by schema-hash test (`SQLITE_SCHEMA_SSOT_SHA256`).

## Required Tables

- `atlas_meta`
- `schema_version`
- `gene_summary` and `genes`
- `transcript_summary` and `transcripts`
- `exons`
- `transcript_exon_map`
- `contigs`
- `dataset_stats`
- `gene_summary_rtree`

## Required Metadata Keys (`atlas_meta`)

- `schema_version`
- `dataset_id`
- `created_by`
- `input_hashes`
- `fasta_sha256`
- `fai_sha256`

## Index Contract

- Gene lookup: `idx_gene_summary_gene_id`
- Gene name lookup: `idx_gene_summary_name`
- Biotype filter: `idx_gene_summary_biotype`
- Stable pagination key: `idx_gene_summary_cover_region` and `idx_genes_order_page`
- Region access: `gene_summary_rtree` + `idx_gene_summary_region`

## Serve-Time SQLite Contract

- Server opens SQLite in read-only immutable URI mode (`mode=ro&immutable=1`).
- `PRAGMA query_only=ON` is always set.
- `busy_timeout=200ms` is set to tolerate short read lock contention without long stalls.
- Connection settings are tuned for read-heavy load (`cache_size`, `mmap_size`, prepared statement cache).
