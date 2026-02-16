# Incident Playbook

## 1. Store Outage
- Switch to cached-only mode if possible.
- Confirm `atlas_store_download_failure_total` trend.
- Verify `/readyz` and serving behavior for already-cached datasets.

## 2. Hot Dataset Missing
- Pin dataset in server config.
- Trigger warm-up and verify checksum/open success.
- Watch `atlas_store_download_p95_seconds` and request 5xx.

## 3. Cache Stampede
- Confirm concurrent misses and download contention.
- Verify single-flight behavior and rate limits are active.
- Temporarily lower heavy query concurrency and increase pinned set.

## 4. Rate-Limit Tuning
- Inspect `atlas_http_requests_total` by status=429 and route.
- Adjust token bucket refill/capacity in controlled increments.
- Re-run load harness and compare p95 + 429/5xx rates.
