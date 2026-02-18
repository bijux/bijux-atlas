# Cache Manager Contract

- Owner: `platform`
- Stability: `stable`

## Purpose

Provide a production-safe local artifact cache for serving datasets with deterministic behavior under load and outages.

## Invariants

- Cache key is artifact identity (`manifest.artifact_hash`, fallback sqlite hash).
- Cache layout is `cache_root/<artifact_hash>/...`.
- Dataset aliases resolve to cache keys via local index entries.
- Server opens cached SQLite in read-only immutable mode only.
- Corrupted artifacts are quarantined and blocked until retry TTL elapses.

## Write Safety

- Download path is atomic: temp write -> checksum verify -> fsync -> rename.
- Artifact lease file prevents concurrent writes to the same artifact directory.
- In-process singleflight prevents duplicate fetches per dataset request fan-in.
- Cross-process safety uses filesystem lease files for shared cache roots.

## Capacity and Eviction

- Disk budget uses high/low watermark policy.
- Eviction keeps pinned datasets.
- Victims are selected by age/size/redownload-cost scoring.

## Outage and Readiness

- Cached-only mode serves cached artifacts and rejects uncached datasets deterministically.
- Store breaker and retry budget prevent tight-loop retry storms.

## Warm and Prefetch

- Startup warmup supports explicit dataset lists.
- Ops warm commands should target pinned/hot datasets.

## Observability

- Internal debug endpoint reports hit/miss, bytes used, and evictions.
- Metrics export download latency/failure and cache pressure counters.
