# Runbook: High Memory

## Symptoms

- RSS growth without stabilization
- pod OOM kills or memory throttling
- latency degradation during sustained load

## Immediate Actions

1. Reduce cache size and dataset count limits.
2. Tighten per-class concurrency limits.
3. Scale replicas horizontally if request load persists.

## Investigation

1. Run memory profile workflow (`docs/runbooks/MEMORY_PROFILE_UNDER_LOAD.md`).
2. Check largest allocation paths and retained buffers.
3. Validate sqlite mmap/cache pragma settings.

## Recovery

1. Apply tuned cache + concurrency config.
2. Redeploy and observe RSS + p95 latency for at least one SLO window.
