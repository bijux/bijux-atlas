# Ingest Docs Index

Ingest pipeline contract:
- Inputs: GFF3 + FASTA + FAI files only.
- Outputs: deterministic `gene_summary.sqlite`, `manifest.json`, `anomaly_report.json`, `qc_report.json`.

Pipeline stages:
- Parse and normalize features from GFF3.
- Validate coordinates against FAI contig lengths.
- Resolve parent-child transcript relationships.
- Materialize deterministic SQLite and metadata artifacts.

Docs:
- [Architecture](ARCHITECTURE.md)
- [Public API](PUBLIC_API.md)
- [Effects policy](EFFECTS.md)
- [Determinism policy](DETERMINISM.md)
- [Ensembl ingest workflow](ENSEMBL_LAYOUT.md)
