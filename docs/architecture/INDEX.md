# Architecture Index

- Owner: `docs-governance`
- Stability: `stable`

## What

Index page for `architecture` documentation.

## Why

Provides a stable section entrypoint.

## Scope

Covers markdown pages directly under this directory.

## Non-goals

Does not duplicate page-level details.

## Contracts

List and maintain links to section pages in this directory.

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
- [Backend Architecture](backend-architecture.md)
- [Component Responsibilities](component-responsibilities.md)
- [Data Access Patterns](data-access-patterns.md)
- [No Serving Writes](no-serving-writes.md)
- [Build/Serve Split](build-serve-split.md)
