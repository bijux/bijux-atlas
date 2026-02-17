# Failure Modes

## Scope

This document defines expected failure behavior for `bijux-atlas-store` operations.

## Modes

- `validation_error`: checksum mismatch, malformed manifest, malformed lock, invalid catalog.
- `conflict`: publish lock contention or attempted overwrite of immutable dataset.
- `network_error`: backend fetch failures, retries exhausted, upstream unavailable.
- `cached_only_mode`: backend is configured to avoid network and artifact is not in cache.
- `not_found`: requested dataset artifact does not exist.
- `io_error`: local filesystem read/write/sync failures.
- `unsupported`: operation not available for backend (for example publish on readonly HTTP backend).
- `internal_error`: unexpected internal invariant failure.

## Publish Safety

- Local filesystem publish is atomic: write temp files, fsync, rename, sync directory.
- `manifest.lock` is mandatory and must match derived artifacts.
- Published datasets are immutable: overwrite attempts fail with conflict.

## Read Safety

- Manifest reads must validate strict schema and lock hash.
- Verified sqlite reads must validate sqlite checksum against manifest.
- Catalog fetch uses ETag where supported and handles `304 Not Modified` without re-parsing.

## Retry Budget

- Network backends use bounded retries and bounded backoff.
- Retry exhaustion must fail fast with stable error class mapping.

## Metrics

- Store instrumentation records download/upload bytes and latency.
- `StoreMetricsCollector` exposes aggregated counters and `failures_by_class`.
