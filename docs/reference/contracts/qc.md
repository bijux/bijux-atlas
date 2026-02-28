# Ingest QC Contract

- Owner: `bijux-atlas-ingest`
- Stability: `stable`

## What

Defines the machine-readable QC report emitted by ingest in `qc.json`.

## Why

Publish, local ops, and CI gates need a stable quality surface independent from human logs.

## Scope

Applies to `qc.json` and compatibility copy `qc_report.json`.

## Contracts

- Schema SSOT: `docs/reference/contracts/schemas/QC_SCHEMA.json`
- Required counters include genes/trantooling/exons/CDS.
- Rejection counts are keyed by stable reason code.
- Top-biotype summary is limited and deterministic.
- Semantics are stable; fields are additive only in schema v1.

## Versioning Rules

- Additive fields do not require consumers to change.
- Breaking field removals/renames require `schema_version` bump.
- CI and publish gates must validate against the pinned schema version.

## Failure modes

Invalid/missing fields fail `atlas ingest-validate` and publish gates.

## How to verify

```bash
make ops-contracts
```
