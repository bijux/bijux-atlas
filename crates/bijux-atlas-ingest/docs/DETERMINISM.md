# Determinism Policy

Rules:
- Stable row ordering by `(seqid, start, end, gene_id)`.
- Stable JSON serialization for manifests/reports.
- Stable hashing based only on file bytes.
- No wall-clock timestamps in ingest outputs.
- Build-time sqlite profile is pinned: `journal_mode=WAL`, `synchronous=OFF`, `locking_mode=EXCLUSIVE`, `temp_store=MEMORY`, fixed `page_size`, fixed `mmap_size`.
- Dataset-pack compaction is deterministic: `ANALYZE` then `VACUUM`, both recorded in `atlas_meta`.
- Build parameters are written to `atlas_meta` for deterministic introspection.
- Schema evolution is forward-only; v1 does not remove existing columns.

Concurrency:
- If parallelism is used, final aggregation and output ordering must remain deterministic.
