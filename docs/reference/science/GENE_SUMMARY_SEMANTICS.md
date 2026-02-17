# Gene Summary Semantics

`gene_summary` is the canonical v1 gene-level materialization.

Fields and meaning:

- `gene_id`: canonical gene identifier from configured gene-id policy.
- `name`: resolved by `GeneNamePolicy` priority list; fallback is reported in anomaly/QC.
- `biotype`: resolved by `BiotypePolicy`; fallback to unknown is reported.
- `seqid,start,end`: normalized and validated against FASTA FAI contig bounds.
- `transcript_count`: number of transcript/mRNA features resolving to the gene under strictness policy.
- `sequence_length`: `end - start + 1` after coordinate validation.

Determinism:

- Stable parse + stable sort + stable insert order.
- Stable JSON and hashing for manifest/report generation.
- No wall-clock-derived fields in artifact content.
