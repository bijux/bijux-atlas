# Store Reference Index

- Owner: `bijux-atlas-store`

## What

Reference entrypoint for artifact store behavior.

## Why

Defines integrity and backend semantics used by ingest and server runtime.

## Scope

Backends, integrity model, ETag caching.

## Non-goals

No server routing behavior.

## Contracts

- [Backends](backends.md)
- [Integrity Model](integrity-model.md)
- [ETag Caching](etag-caching.md)

## Failure modes

Weak integrity validation can publish corrupted artifacts.

## How to verify

```bash
$ cargo nextest run -p bijux-atlas-store
```

Expected output: backend conformance and integrity tests pass.

## See also

- [Datasets Reference](../datasets/INDEX.md)
- [Registry Reference](../registry/INDEX.md)
- [Contracts Artifacts](../../contracts/artifacts/manifest-contract.md)
