# Quickstart Index

- Owner: `docs-governance`

## What

Canonical onboarding root is `docs/START_HERE.md`.

Use this directory only for supporting quickstart references.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

- [Ops Local Full Quickstart](ops-local-full.md)

## Failure modes

Missing index links create orphan docs.

## How to verify

```bash
$ make docs
```

Expected output: docs build and docs-structure checks pass.

## See also

- [Docs Home](../index.md)
- [Naming Standard](../_style/naming-standard.md)
- [Terms Glossary](../_style/terms-glossary.md)
