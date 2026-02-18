# GFF3 Acceptance Contract (v1)

- Owner: `bijux-atlas-ingest`
- Stability: `stable`

## In-Scope Features

Accepted and processed in v1:

- `gene`
- transcript features from `TranscriptTypePolicy` (default: `transcript`, `mRNA`, `mrna`)
- `exon`
- `CDS`

Unknown feature types are governed by `UnknownFeaturePolicy`:

- `IgnoreWithWarning` (default): ignored, recorded in anomaly report
- `Reject`: strict ingest failure

## ID Uniqueness Policy

`FeatureIdUniquenessPolicy` defines how `ID` collisions are treated:

- `Reject` (default): duplicate feature IDs fail in strict mode (gene duplicates are handled by duplicate-gene policy)
- `NamespaceByFeatureType`: uniqueness is checked per feature type
- `NormalizeAsciiLowercaseReject`: IDs are lowercased before uniqueness checks, then rejected on collision

## Parent Policy

- Missing `Parent` on transcript: reject in strict mode; anomaly in non-strict modes.
- Transcript with multiple `Parent`: reject in strict mode; anomaly in non-strict modes.
- Missing `Parent` on `exon`/`CDS`: reject in strict mode; anomaly in non-strict modes.
- Multi-parent `exon`/`CDS`: reject in strict mode.
- Orphan children (child references unknown transcript/gene): anomaly and strict-mode rejection where applicable.
- Cycles in `Parent` graph: detected before extraction; strict-mode rejection.

## Missing Required Fields

Parser rejects malformed rows with explicit reasons, including:

- missing `seqid`
- missing `feature_type`
- invalid coordinate span (`start >= 1`, `start <= end`)
- invalid column count (must be 9)

Feature-level required attribute checks:

- `gene` requires `ID`
- transcript features require transcript ID per `TranscriptIdPolicy` and `Parent`
- `exon`/`CDS` require `Parent`

## Attribute Canonicalization

- Attributes parsed from semicolon-separated `key=value` pairs.
- Values are trimmed, surrounding quotes removed, and percent-decoded.
- Duplicate attribute keys are recorded as anomalies (last value wins).
- Attribute token explosion is rejected by parser bound.

## Name/Biotype/Contig/Coordinate Rules

- Gene name extraction: `GeneNamePolicy` key priority (default includes `gene_name`, `Name`, ...).
- Transcript ID extraction: `TranscriptIdPolicy` key priority (default includes `ID`, `transcript_id`, `transcriptId`).
- Gene biotype extraction: `BiotypePolicy` key priority with explicit `unknown_value` fallback.
- Transcript biotype uses `transcript_biotype`, then `biotype`, then `gene_biotype`.
- Contigs are canonicalized via `SeqidNormalizationPolicy` aliases.
- Coordinates are validated as 1-based inclusive and bounded by FAI contig lengths.
