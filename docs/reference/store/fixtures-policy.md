# Fixture Policy

CI fixtures:

- `crates/bijux-atlas-ingest/tests/opssuite/ops/datasets/fixtures/minimal`
- `crates/bijux-atlas-ingest/tests/opssuite/ops/datasets/fixtures/edgecases`
- `crates/bijux-atlas-ingest/tests/opssuite/ops/datasets/fixtures/contigs`

Non-CI fixture:

- medium dataset is fetched via `make fetch-fixtures` using pinned URL + sha256 in `ops/datasets/fixtures/medium/v1/manifest.lock`.

Golden query snapshots:

- Query definitions: `ops/datasets/fixtures/medium/v1/api-list-queries.v1.json`.
- Snapshot output (manual): `ops/datasets/fixtures/medium/v1/api-list-responses.v1.json`.

## What

Reference definition for this topic.

## Why

Defines stable semantics and operational expectations.

## Scope

Applies to the documented subsystem behavior only.

## Non-goals

Does not define unrelated implementation details.

## Contracts

Normative behavior and limits are listed here.

## Failure modes

Known failure classes and rejection behavior.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
