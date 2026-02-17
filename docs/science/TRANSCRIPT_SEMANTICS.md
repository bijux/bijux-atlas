# Transcript Semantics

`transcript_summary` captures transcript-level records accepted by `TranscriptTypePolicy`.

v1 interpretation:

- Accepted transcript feature types are policy-driven (`transcript`, `mRNA`, variants).
- `parent_gene_id` must resolve to an existing gene to be served.
- `exon_count` and `total_exon_span` are accumulated from exon features by transcript Parent.
- `cds_present` is true if at least one CDS feature references the transcript.

Anomaly and strictness behavior:

- Missing Parent, multiple Parent, unresolved Parent, or cyclic Parent graphs are recorded.
- Orphan transcripts are reported explicitly in anomaly/QC outputs.
- Attribute fallbacks (gene id/name/biotype) are recorded; strict warning mode can fail ingest.
