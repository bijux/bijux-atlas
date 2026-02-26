# Node-Local Shared Cache Profile

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

This is an advanced deployment profile, not the default.

## Overview
- Run a DaemonSet that exposes a node-local cache path for atlas datasets.
- Mount the same hostPath (or local PV) into atlas pods as read/write cache root.
- Keep normal remote store as source-of-truth; node-local cache is an accelerator.

## Constraints
- Cache contents are best-effort and can be evicted by node lifecycle events.
- Keep `cached_only_mode=false` in this profile unless the node cache is pre-warmed.
- Enforce `max_concurrent_downloads` to avoid herd downloads on node restart.

## Suggested Settings
- `ATLAS_CACHE_ROOT=/var/lib/bijux-atlas/cache`
- `ATLAS_MAX_CONCURRENT_DOWNLOADS=2`
- `ATLAS_STARTUP_WARMUP_LIMIT=4`
- `ATLAS_STARTUP_WARMUP_JITTER_MAX_MS=30000`
- `ATLAS_SQLITE_MMAP_BYTES=268435456`

## SSD Comparison Checklist
- Capture baseline from `/metrics`:
  - `bijux_http_request_latency_p95_seconds`
  - `bijux_store_download_p95_seconds`
  - `bijux_dataset_disk_usage_bytes`
- Repeat after node-local SSD rollout using same k6 suites:
  - `ops/load/k6/suites/warm-steady.js`
  - `ops/load/k6/suites/regional-spike-10x-60s.js`
- Keep query mix and dataset set identical between runs.
## Referenced chart values keys

- `values.cache`
- `values.resources`
- `values.server`

## See also

- `ops-ci`
