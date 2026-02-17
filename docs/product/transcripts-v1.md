# Transcripts v1

This document defines transcript behavior in Atlas v1.

## Scope

Atlas v1 includes transcript summaries derived from GFF3 transcript/mRNA features.

Fields include:

- `transcript_id`
- `parent_gene_id`
- `transcript_type`
- `biotype` (when available)
- `seqid`, `start`, `end`
- `exon_count`
- `total_exon_span`
- `cds_present`

## Meaning of `transcript_count`

`gene_summary.transcript_count` is the number of transcript features whose `Parent` resolves to that gene under current strictness policy.

It is not a canonical biology claim across all annotation sources; it is a deterministic count under the configured ingest policies.

## Ordering Rules

Transcript list endpoints return stable ordering:

1. `seqid` ascending
2. `start` ascending
3. `transcript_id` ascending (tie-breaker)

## Canonical Transcript Policy

Atlas v1 does **not** define canonical transcript selection.

Canonical transcript ranking/selection is a v2 policy placeholder and explicit non-goal for v1.
