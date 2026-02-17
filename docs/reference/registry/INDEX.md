# Registry Reference Index

- Owner: `bijux-atlas-store`

## What

Reference entrypoint for multi-registry behavior.

## Why

Federation semantics must remain deterministic across stores.

## Scope

Federation semantics, deterministic merge, conflict resolution.

## Non-goals

No transport-specific backend implementation detail.

## Contracts

- [Federation Semantics](federation-semantics.md)
- [Deterministic Merge](deterministic-merge.md)
- [Conflict Resolution](conflict-resolution.md)

## Failure modes

Non-deterministic merge order can change visible datasets between pods.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-store federation
```

Expected output: federation merge tests pass deterministically.

## See also

- [Store Reference](../store/INDEX.md)
- [Contracts Compatibility](../../contracts/compatibility.md)
- [Runbook Federation](../../operations/runbooks/registry-federation.md)
