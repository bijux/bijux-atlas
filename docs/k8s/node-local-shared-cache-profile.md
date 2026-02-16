# Node-Local Shared Cache Profile

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
- `ATLAS_SQLITE_MMAP_BYTES=268435456`
