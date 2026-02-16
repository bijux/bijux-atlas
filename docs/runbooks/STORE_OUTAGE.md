# Runbook: Store Outage

## Symptoms

- Spike in `bijux_store_download_failure_total`
- Growth in `bijux_store_breaker_open_total` or `bijux_store_retry_budget_exhausted_total`
- Increased `503` on dataset-open paths
- cache misses cannot recover

## Immediate Actions

1. Enable cached-only mode (`ATLAS_CACHED_ONLY_MODE=true`) if cache has critical datasets.
2. Increase pinned datasets to protect known hot datasets.
3. Reduce heavy query concurrency and tighten rate limits.

## Investigation

1. Validate store endpoint/network health.
2. Verify auth credentials and token expiry.
3. Check retry/backoff config (`ATLAS_STORE_RETRY_ATTEMPTS`, `ATLAS_STORE_RETRY_BASE_MS`).
4. Check cache-manager guards (`ATLAS_STORE_RETRY_BUDGET`, `ATLAS_STORE_BREAKER_FAILURE_THRESHOLD`, `ATLAS_STORE_BREAKER_OPEN_MS`).

## Recovery

1. Restore store access.
2. Disable cached-only mode.
3. Watch `bijux_store_download_failure_total` and `bijux_dataset_hits/misses` normalize.
