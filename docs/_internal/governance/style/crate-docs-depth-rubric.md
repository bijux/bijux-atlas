# Crate Docs Depth Rubric

- Owner: `docs-governance`

## What

Defines minimum depth and section requirements for crate documentation.

## Why

Prevents shallow crate docs and keeps extension/test guidance consistent.

## Scope

Applies to `crates/*/docs/*.md` and `crates/*/README.md`.

## Non-goals

Does not prescribe crate-internal implementation style.

## Contracts

Required section headings for major crate docs:

- `architecture.md`: `## Purpose`, `## Invariants`, `## Boundaries`, `## Failure modes`, `## How to test`
- `effects.md`: `## Purpose`, `## Invariants`, `## Boundaries`, `## Failure modes`, `## How to test`
- `public-api.md`: `## Purpose`, `## Invariants`, `## Boundaries`, `## Failure modes`, `## How to test`
- `testing.md`: `## Purpose`, `## Invariants`, `## Boundaries`, `## Failure modes`, `## How to test`
- `contracts.md` (when present): above sections plus `## Versioning`
- `failure-modes.md` (required for server/store/ingest): above sections

## Failure modes

Missing sections or shallow docs hide invariants and extension risks.

## How to verify

```bash
$ bin/bijux-atlas docs crate-docs-contract-check --report text
```

Expected output: crate docs contract check passes.

## See also

- [Structure Templates](structure-templates.md)
- [Docs style](../docs-style.md)
- [Repository layout](../../../development/repo-layout.md)
