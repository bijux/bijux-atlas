# Failure Modes

## Store Unavailable

- Catalog/store access failures trip store circuit-breaker thresholds.
- Server enters bounded retry/cooldown behavior.
- In cached-only mode, cached datasets continue serving.

## Dataset Corruption Or Verification Failure

- Checksum verification failure rejects dataset open.
- Dataset is not promoted to active cache.
- Repeated failures increment per-dataset failure counters and may quarantine opens.

## Overload

- Query classes (`cheap`, `medium`, `heavy`) are isolated by bulkheads.
- Heavy paths can be shed while cheap paths remain available.
- Response-size and request-timeout guards prevent tail-amplification.

## Slow Queries

- SQLite progress-handler timeout aborts long-running queries.
- Timeout surfaces stable error codes and telemetry labels.

## Catalog Drift Or Staleness

- ETag/backoff catalog cache avoids hot-loop polling.
- Last known good catalog remains active until refresh succeeds.

## Operator Guidance

- Prefer cached-only mode during known store incidents.
- Use warm-up jobs for pinned datasets before reopening traffic.
- Monitor cache hit-rate, circuit-breaker state, and overload metrics together.
