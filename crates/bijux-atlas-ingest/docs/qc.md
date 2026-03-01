# QC Policy

QC is emitted for every ingest run.

- Primary machine artifact: `qc.json`
- Compatibility copy: `qc_report.json`
- Schema contract: `docs/reference/contracts/schemas/QC_SCHEMA.json`

## Severity

- `INFO`: informational counts.
- `WARN`: anomalies that keep ingest successful in lenient/report-only modes.
- `ERROR`: reserved for future fatal QC classifications.

## Tracked anomaly classes

- missing parents
- missing transcript parents
- multiple transcript parents
- unknown contigs
- overlapping ids
- duplicate gene ids
- overlapping gene ids across contigs
- orphan transcripts
- parent cycles
- attribute fallbacks

## Report-only mode

`atlas ingest --report-only` writes QC + anomaly outputs without producing SQLite or manifest artifacts.

## Stability

QC semantics are stable in schema v1. Fields are additive-only; breaking changes require schema version bump.
