# Failure Modes

- Owner: `bijux-atlas-ingest`

## Purpose

Describes failure classes and expected mitigation behavior.

## Invariants

- Failures return stable machine-readable errors.
- Partial operations are never published as successful.

## Boundaries

- Covers runtime/storage/ingest failures owned by this crate.

## Failure modes

- Invalid input artifacts or contract violations.
- Resource limits (timeouts, disk, memory) and dependency outages.

## How to test

```bash
$ cargo nextest run -p bijux-atlas-ingest failure
```

Expected output: failure-path tests pass.

```bash
$ make dev-test-all
```

Expected output: workspace regression suite remains green.
