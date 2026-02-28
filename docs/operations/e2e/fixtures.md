# E2E Fixtures

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines pinned fixture packs used by e2e workflows.

## Why

Keeps fixture provenance and checksums deterministic.

## Scope

`ops/e2e/fixtures/README.md` and fixture lock usage.

## Non-goals

Does not define ingest artifact contract.

## Contracts

- Fixture lock is authoritative for URLs and SHA256.
- E2E scripts must consume pinned fixture entries only.

## Failure modes

Unpinned fixtures introduce nondeterministic failures.

## How to verify

```bash
$ make fetch-real-datasets
```

Expected output: fixture downloads pass checksum validation.

## See also

- [E2E Overview](overview.md)
- [Fixtures Reference](../../reference/index.md)
- [Reference Schemas](../../reference/schemas.md)
