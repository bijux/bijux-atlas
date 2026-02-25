# Medium Fixture (Non-CI)

This dataset is not committed in full to this repository. The archive source of truth is a release asset in the fixtures distribution channel.

Fetch via:

- `make fetch-fixtures`

Golden query snapshots for this fixture:

- `ops/datasets/fixtures/medium/v1/api-list-queries.v1.json`
- `ops/datasets/fixtures/medium/v1/api-list-responses.v1.json`

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
