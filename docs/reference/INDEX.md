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
- Generated cross-surface references are published in this section and must be refreshed by `bijux dev atlas docs reference generate --allow-subprocess --allow-write`.

## Failure modes

Missing links create orphan technical references.

## How to verify

```bash
$ make docs-validate
```

Expected output: docs checks pass with no orphan pages.

## See also

- [Docs Home](../index.md)
- [Architecture](../architecture/INDEX.md)
- [Terms Glossary](../_style/terms-glossary.md)
- [Commands Reference](commands.md)
- [Schemas Reference](schemas.md)
- [Configs Reference](configs.md)
- [Make Targets Reference](make-targets.md)
