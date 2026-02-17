# Ingest Contract

## Inputs

- `GFF3` annotation file.
- `FASTA` reference file.
- `FAI` index for contig bounds.

## Outputs

- Normal mode: `gene_summary.sqlite`, `manifest.json`, `anomaly_report.json`, `qc_report.json`.
- Report-only mode: `anomaly_report.json` and `qc_report.json` only.

## Guarantees

- Deterministic ordering and checksums for identical inputs/config.
- Strict schema output for manifest and anomaly reports.
- Parent graph validation and anomaly reporting.
- Seqid coordinate validation against `.fai`.

## Non-goals

- No network reads/writes.
- No hidden mutable state.
