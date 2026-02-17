# Error Code Registry

Concept IDs: concept.error-codes

- Owner: `docs-governance`

Canonical page: [`docs/contracts/errors.md`](../../contracts/errors.md)

## What

Pointer page for error code registry semantics.

## Why

Prevents error-code contract duplication outside contracts section.

## Scope

Machine error codes used by API and CLI surfaces.

## Non-goals

Does not define new codes or schemas.

## Contracts

Error code definitions are sourced from the canonical page.

## Failure modes

Divergent code lists can break clients and automation.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Error Codes Contract](../../contracts/errors.md)
- [Contracts Index](../../contracts/INDEX.md)
- [Registry Index](INDEX.md)
