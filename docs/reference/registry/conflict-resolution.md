# Conflict Resolution

Concept IDs: concept.registry-federation

- Owner: `bijux-atlas-store`

Canonical page: [`docs/reference/registry/federation-semantics.md`](federation-semantics.md)

## What

Pointer page for conflict handling in federated registries.

## Why

Keeps conflict semantics aligned with canonical federation contract.

## Scope

Priority and shadowing outcomes for overlapping dataset identities.

## Non-goals

Does not add independent conflict policy statements.

## Contracts

Conflict outcomes are defined by the canonical federation page.

## Failure modes

Duplicate conflict policy text can diverge and break operators.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Federation Semantics](federation-semantics.md)
- [Deterministic Merge](deterministic-merge.md)
- [Registry Index](INDEX.md)
