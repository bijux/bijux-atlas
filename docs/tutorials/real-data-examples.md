# Tutorial Real Data Examples

This tutorial set uses 10 real public datasets from NCBI and ENA, with no synthetic placeholders.

## Dataset Sources

| Run ID | Dataset | Source | Size (bytes) |
|---|---|---|---:|
| rdr-001-genes-baseline | genes-baseline | NCBI efetch (NC_045512.2 FASTA) | 30429 |
| rdr-002-transcripts-baseline | transcripts-baseline | NCBI efetch (NC_045512.2 GenBank) | 78517 |
| rdr-003-variants-mini | variants-mini | ENA browser FASTA (AB000263.1) | 462 |
| rdr-004-expression-medium | expression-medium | ENA browser EMBL (AB000263.1) | 2904 |
| rdr-005-population-medium | population-medium | ENA portal read_run report (SRR390728) | 251 |
| rdr-006-phenotype-medium | phenotype-medium | ENA portal read_run report (ERR194147) | 330 |
| rdr-007-assembly-large-sample | assembly-large-sample | NCBI efetch (U00096.3 FASTA) | 4708032 |
| rdr-008-annotation-large-sample | annotation-large-sample | NCBI RefSeq FTP genomic GFF | 406858 |
| rdr-009-clinvar-large-sample | clinvar-large-sample | NCBI RefSeq FTP protein FAA | 902395 |
| rdr-010-combined-release | combined-release | ENA browser FASTA (K03455.1) | 9999 |

All datasets are below 1 GB. More than half are below 100 MB.

## Execute End-to-End

```bash
cargo run -p bijux-dev-atlas -- tutorials real-data run-all --profile local --format json
```

## Verify Per-Run Outputs

Each run should have these files under `artifacts/tutorials/runs/<run_id>/`:

- `ingest-report.json`
- `dataset-summary.json`
- `query-results-summary.json`
- `evidence-bundle.json`
- `manifest.json`
- `bundle.sha256`

## Downloaded Dataset Artifacts

Downloaded files are stored in:

- `artifacts/tutorials/datasets/<dataset>/dataset.bin`
- `artifacts/tutorials/datasets/<dataset>/sha256sums.txt`
- `artifacts/tutorials/datasets/<dataset>/provenance.json`

Cached run inputs are stored in:

- `artifacts/tutorials/cache/<dataset>/dataset.bin`
