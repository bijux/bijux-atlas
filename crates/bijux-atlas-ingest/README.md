# bijux-atlas-ingest

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

- `docs/INGEST_CONTRACT.md`
- `docs/ARTIFACT_OUTPUT_CONTRACT.md`
- `docs/DETERMINISM.md`
