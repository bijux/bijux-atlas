# Artifact Output Contract

## Produced Artifacts

For each dataset ingest, outputs are written under:

`release=<r>/species=<s>/assembly=<a>/derived/`

Primary artifacts:
- `manifest.json`
- `gene_summary.sqlite`
- `manifest.lock`
- `anomaly_report.json`
- `qc_report.json`
- `release_gene_index.json`

Optional artifacts:
- `normalized_features.jsonl.zst` (debug/replay)
- sharding catalog and shard sqlite files (when sharding is enabled)

## Schema References

- Manifest and catalog schemas: `bijux-atlas-model`
- SQLite schema SSOT: `sql/schema_v4.sql`

## Determinism Guarantees

- parse/decode stage is pure
- deterministic ordering is applied before persistence
- hashing uses centralized content-address module
- timestamp policy defaults to deterministic behavior
