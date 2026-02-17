# Ingest Docs Index

Ingest pipeline contract:
- Inputs: GFF3 + FASTA + FAI files only.
- Outputs: deterministic `gene_summary.sqlite`, `manifest.json`, `anomaly_report.json`, `qc_report.json`.
- Optional shard mode outputs: `catalog_shards.json` plus `gene_summary.<shard>.sqlite`.

Pipeline stages:
- Parse and normalize features from GFF3.
- Validate coordinates against FAI contig lengths.
- Resolve parent-child transcript relationships.
- Materialize deterministic SQLite and metadata artifacts.
- Support report-only QC/anomaly generation without writing SQLite/manifest.
- Optional shard materialization: `per-seqid` or bounded partition count for large datasets.
- Strict warning mode (`--strict`) can fail ingest when QC WARN items are present.

Docs:
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Effects policy](EFFECTS.md)
- [Determinism policy](DETERMINISM.md)
- [Ingest contract](INGEST_CONTRACT.md)
- [QC policy](QC.md)
- [Ensembl ingest workflow](ENSEMBL_LAYOUT.md)
