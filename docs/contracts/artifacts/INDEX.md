# Contracts Artifacts Index

- Owner: `docs-governance`

## What

Index page for `contracts/artifacts` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

- [Manifest Contract](manifest-contract.md)
- [SQLite Schema Contract](sqlite-schema-contract.md)
- [SQLite Schema Evolution Strategy](sqlite-schema-evolution-strategy.md)

## Examples

- `ARTIFACT_SCHEMA.json`

```json
{
  "schema_version": "v1",
  "dataset_id": {
    "release": "110",
    "species": "homo_sapiens",
    "assembly": "GRCh38"
  }
}
```

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
```

Expected output: docs build and docs-structure checks pass.

## See also

- [Docs Home](../../index.md)
- [Naming Standard](../../_style/naming-standard.md)
- [Terms Glossary](../../_style/terms-glossary.md)
