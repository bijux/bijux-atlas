# Ingestion Reference Index

- Owner: `bijux-atlas-ingest`

## What

Reference entrypoint for ingest semantics and determinism.

## Why

Ingest correctness defines downstream API trust.

## Scope

Parser semantics, QC report schema, determinism, supported layouts.

## Non-goals

No manual curation workflows.

## Contracts

- [Parser Semantics](parser-semantics.md)
- [QC Report Schema](qc-report-schema.md)
- [Determinism](determinism.md)
- [Supported Layouts](supported-layouts.md)

## Failure modes

Non-deterministic ingest invalidates signatures and compatibility.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-ingest
```

Expected output: parser/QC/determinism tests pass.

## See also

- [Contracts Artifacts](../../contracts/artifacts/manifest-contract.md)
- [Product What Is Atlas](../../product/what-is-atlas.md)
- [Datasets Reference](../datasets/INDEX.md)
