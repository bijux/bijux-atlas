# Layering Architecture

- Owner: `platform-architecture`

## What

Index for layer-boundary contracts in `docs/architecture/layering/`.

## Why

Makes layer ownership and "fixes must live in the right layer" policy explicit and reviewable.

## Scope

Boundary rules and layer responsibilities for ops stack, k8s, e2e, observability, and load areas.

## Non-goals

Does not replace implementation-level runbooks in `ops/` or scenario-level e2e docs.

## Contracts

- [Boundary Rules](boundary-rules.md)

## Failure modes

Missing or stale boundary docs allow cross-layer fixups and hidden coupling.

## How to verify

```bash
make docs/all
```

Expected output: docs lint/build/tests pass.

## See also

- [Architecture Index](../INDEX.md)
- [What E2E Is Not](../../operations/e2e/what-e2e-is-not.md)
