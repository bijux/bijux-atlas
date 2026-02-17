# Reference Index

- Owner: `docs-governance`

## What

Canonical entrypoint for all technical reference sections.

## Why

Reference content must be navigable from one stable index page.

## Scope

Datasets, querying, performance, security, registry, store, ingestion, evolution, fixtures, and science semantics.

## Non-goals

No product positioning or tutorial content.

## Contracts

- Each reference subsection must maintain its own `INDEX.md`.

## Failure modes

Missing links create orphan technical references.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass with no orphan pages.

## See also

- [Docs Home](../index.md)
- [Architecture](../architecture/INDEX.md)
- [Terms Glossary](../_style/terms-glossary.md)
