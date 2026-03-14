# Determinism Policy

Rules:
- Stable row ordering by `(seqid, start, end, gene_id)`.
- Stable JSON serialization for manifests/reports.
- Stable hashing based only on file bytes.
- No wall-clock timestamps in ingest outputs.
- Build-time sqlite profile is pinned: `journal_mode=WAL`, `synchronous=OFF`, `locking_mode=EXCLUSIVE`, `temp_store=MEMORY`, fixed `page_size`, fixed `mmap_size`.
- Dataset-pack compaction is deterministic: `ANALYZE` then `VACUUM`, both recorded in `atlas_meta`.
- Build parameters are written to `atlas_meta` for deterministic introspection.
- Manifest includes reproducibility proof fields: `ingest_toolchain` and `ingest_build_hash`.
- Schema evolution is forward-only; v1 does not remove existing columns.
- Artifact hash definition (v1): `sha256(sqlite_bytes)` with manifest-side checksums and signatures; `created_at` is excluded from determinism guarantees.
- Cross-platform policy: tiny fixture golden hashes are pinned and validated in CI for Linux/macOS runners when available.

Concurrency:
- If parallelism is used, final aggregation and output ordering must remain deterministic.
