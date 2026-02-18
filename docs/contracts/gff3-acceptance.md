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
- Values are trimmed, surrounding quotes removed, percent-decoded, then NFC-normalized.
- Hidden/control characters are forbidden in ID/name fields and rejected (`GFF3_FORBIDDEN_HIDDEN_CHAR`).
- ID-like keys (`ID`, `Parent`, `gene_id`, `transcript_id`, `transcriptId`, `protein_id`, `exon_id`) must not contain whitespace (`GFF3_INVALID_ID_WHITESPACE`).
- Duplicate attribute keys are recorded as anomalies (last value wins).
- Attribute token explosion is rejected by parser bound.

## Name/Biotype/Contig/Coordinate Rules

- Gene name extraction: `GeneNamePolicy` key priority (default includes `gene_name`, `Name`, ...).
- Transcript ID extraction: `TranscriptIdPolicy` key priority (default includes `ID`, `transcript_id`, `transcriptId`).
- Gene biotype extraction: `BiotypePolicy` key priority with explicit `unknown_value` fallback.
- Transcript biotype uses `transcript_biotype`, then `biotype`, then `gene_biotype`.
- Contigs are canonicalized via `SeqidNormalizationPolicy` aliases.
- Strand must be one of `+`, `-`, `.`.
- CDS phase must be one of `0`, `1`, `2`, `.`.
- Canonicalized contig collisions are rejected in strict mode by default.
- Coordinates are validated as 1-based inclusive and bounded by FAI contig lengths.

## Duplicate Entity Policies

- Duplicate gene IDs: governed by `DuplicateGeneIdPolicy` (`Fail` or deterministic dedupe).
- Duplicate transcript IDs: governed by `DuplicateTranscriptIdPolicy` (`Reject` or deterministic dedupe).
- Extraction is order-independent: valid outputs must not depend on row grouping/order in source GFF3.

## Rejections Report

Anomaly report includes structured rejection entries:

- `line`: source line number (or `0` for aggregate duplicate-id cases)
- `code`: stable rejection code
- `sample`: source line excerpt or identifier sample

Current rejection codes:

- `GFF3_MISSING_REQUIRED_FIELD`
- `GFF3_INVALID_STRAND`
- `GFF3_INVALID_PHASE`
- `GFF3_MISSING_TRANSCRIPT_ID`
- `GFF3_MISSING_PARENT`
- `GFF3_MULTI_PARENT_TRANSCRIPT`
- `GFF3_MULTI_PARENT_CHILD`
- `GFF3_UNKNOWN_FEATURE`
- `GFF3_DUPLICATE_TRANSCRIPT_ID`
