# Dataset Lifecycle

State machine:

1. Ingest: parse/validate raw inputs and compute derived artifacts.
2. Publish artifact: atomically publish manifest + SQLite + checksums.
3. Promote: add dataset entry to `catalog.json`.
4. Latest alias update: update `latest.alias.json` only for promoted datasets.
5. Serve: API reads only published immutable artifacts.
6. Rollback: remove catalog pointer to bad release; artifacts remain immutable.
7. Deprecate: mark dataset as deprecated in catalog without mutating historical artifacts.

## What

Reference definition for this topic.

## Why

Defines stable semantics and operational expectations.

## Scope

Applies to the documented subsystem behavior only.

## Non-goals

Does not define unrelated implementation details.

## Contracts

- Promotion is pointer-only: no artifact mutation.
- Latest alias update is promotion-gated.
- Rollback reverts catalog pointer, never rewrites dataset files.
- Immutable release rule: fixes ship as new dataset identity.

## Failure modes

Known failure classes and rejection behavior.

## How to verify

```bash
$ make ops-catalog-validate
```

Expected output: catalog validates and lifecycle commands pass.

## See also

- [Reference Index](INDEX.md)
- [Contracts Index](../../contracts/contracts-index.md)
- [Terms Glossary](../../_style/terms-glossary.md)
