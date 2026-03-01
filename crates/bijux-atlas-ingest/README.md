# bijux-atlas-ingest

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

Ingestion pipeline for converting GFF3/FASTA/FAI inputs into atlas artifacts.

## Supported Formats

- GFF3 features (genes, transcripts, exons, CDS)
- FASTA sequence file
- FASTA index (FAI)

## Deterministic Guarantees

- decode/parse is separated from write/store effects
- ingest plans are explicit (`IngestJob`)
- deterministic ordering for extracted records before persistence
- centralized input hashing/content-address policy
- deterministic timestamp policy by default

## Common Failure Modes

- malformed GFF3 coordinates or missing required fields
- missing FAI index without explicit auto-generation opt-in
- unknown contigs under strict mode
- checksum mismatches in generated artifacts

## Reference Docs

- `docs/ingest-contract.md`
- `docs/artifact-output-contract.md`
- `docs/determinism.md`
