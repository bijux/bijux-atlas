# Fixture Policy

CI fixtures:

- `crates/bijux-atlas-ingest/tests/fixtures/minimal`
- `crates/bijux-atlas-ingest/tests/fixtures/edgecases`
- `crates/bijux-atlas-ingest/tests/fixtures/contigs`

Non-CI fixture:

- medium dataset is fetched via `make fetch-fixtures` using pinned URL + sha256 in `fixtures/medium/manifest.lock`.

Golden query snapshots:

- Query definitions: `fixtures/medium/golden_queries.json`.
- Snapshot output (manual): `fixtures/medium/golden_snapshot.json`.

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
