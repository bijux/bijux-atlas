# Public API

- Owner: `bijux-atlas-ingest`
- Stability reference: [Stability Levels](../../../docs/_internal/governance/style/stability-levels.md)

## Purpose

Defines the current public surface exported by `bijux-atlas-ingest`.

## Invariants

- Public ingest outputs remain deterministic for identical inputs.
- Public API changes are documented here before release.

## Boundaries

- This crate exposes ingest entrypoints and ingest result types.
- Runtime serving and HTTP behavior are out of scope.

## Failure modes

- Invalid source files produce deterministic validation errors.
- Policy violations reject ingest with stable error categories.

## Public types

- `IngestOptions`
- `IngestInputs`
- `IngestJob`
- `IngestResult`
- `IngestError`
- `InputHashes`
- `IngestEvent`
- `IngestLog`
- `IngestStage`

## How to test

```bash
$ cargo nextest run -p bijux-atlas-ingest
```

Expected output: ingest unit and integration tests pass.

```bash
$ cargo test -p bijux-atlas-ingest --doc
```

Expected output: doc tests pass.
