# Deterministic Merge

Concept IDs: concept.registry-federation

- Owner: `bijux-atlas-store`

Canonical page: [`docs/reference/registry/federation-semantics.md`](federation-semantics.md)

## What

Pointer page for deterministic merge behavior in federated registries.

## Why

Avoids duplicate merge semantics definitions.

## Scope

Merge ordering, stability, and priority semantics.

## Non-goals

Does not define backend transport implementation.

## Contracts

Normative merge semantics are defined in the canonical page.

## Failure modes

Inconsistent merge definitions create non-deterministic catalog views.

## How to verify

```bash
$ make docs
```

Expected output: docs checks pass.

## See also

- [Federation Semantics](federation-semantics.md)
- [Conflict Resolution](conflict-resolution.md)
- [Registry Index](INDEX.md)
