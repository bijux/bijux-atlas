# Ingest Contract

## Inputs

- `GFF3` annotation file.
- `FASTA` reference file.
- `FAI` index for contig bounds (required in production ingest).
- Dev-only option can auto-generate `.fai` from FASTA.

## Outputs

- Normal mode: `gene_summary.sqlite`, `manifest.json`, `anomaly_report.json`, `qc_report.json`.
- Report-only mode: `anomaly_report.json` and `qc_report.json` only.

## Guarantees

- Deterministic ordering and checksums for identical inputs/config.
- Strict schema output for manifest and anomaly reports.
- Parent graph validation and anomaly reporting.
- Seqid coordinate validation against `.fai`.
- All GFF3 contigs must exist in `.fai`; unknown contigs fail in strict mode.
- `sequence_length` in v1 means genomic span length (`end-start+1`) for both gene and transcript rows.
- Optional compute mode can emit transcript spliced length (exon union) and CDS span length.
- SQLite stores a `contigs` table (`name`, `length`, optional `gc_fraction`, `n_fraction`).

## Non-goals

- No network reads/writes.
- No hidden mutable state.
