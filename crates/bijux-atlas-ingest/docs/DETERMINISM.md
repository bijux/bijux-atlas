# Determinism Policy

Rules:
- Stable row ordering by `(seqid, start, end, gene_id)`.
- Stable JSON serialization for manifests/reports.
- Stable hashing based only on file bytes.
- No wall-clock timestamps in ingest outputs.

Concurrency:
- If parallelism is used, final aggregation and output ordering must remain deterministic.
