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
- [Architecture](architecture.md)
- [Public API](public-api.md)
- [Effects policy](effects.md)
- [Determinism policy](determinism.md)
- [Ingest contract](ingest-contract.md)
- [Artifact output contract](artifact-output-contract.md)
- [QC policy](qc.md)
- [Ensembl ingest workflow](ensembl-layout.md)

- [How to test](testing.md)
- [How to extend](#how-to-extend)

## API stability

Public API is defined only by `docs/public-api.md`; all other symbols are internal and may change without notice.

## Invariants

Core invariants for this crate must remain true across releases and tests.

## Failure modes

Failure modes are documented and mapped to stable error handling behavior.

## How to extend

Additions must preserve crate boundaries, update `docs/public-api.md`, and add targeted tests.

- Central docs index: docs/index.md
- Crate README: ../README.md
