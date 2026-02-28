# Ingest Format Contracts

Owner: `bijux-atlas-ingest`  
Type: `guide`  
Audience: `contributor`  
Reason to exist: document ingest-specific format acceptance rules that are implementation-facing, not public API reference.

## FASTA-Derived Metrics

- Feature flag: `fasta_scanning_enabled` defaults to `false`.
- With `compute_contig_fractions=true`, contig GC/N fractions are computed deterministically and stored in SQLite `contigs`.
- Scan order is deterministic and bounded by `fasta_scan_max_bases`.

## GFF3 Acceptance

- Accepted v1 features: `gene`, transcript features from `TranscriptTypePolicy`, `exon`, `CDS`.
- Unknown features are handled by `UnknownFeaturePolicy`.
- Parent relationships, required fields, strand/phase values, and coordinate integrity are validated with strict-mode rejection.
- Duplicate ID handling follows explicit duplicate policies for genes and transcripts.

## Validation

Run ingest contract checks through:

```bash
make ssot-check
```
