# Startup Warm-Cache Strategy

Atlas supports startup warm-cache through environment configuration:

- `ATLAS_STARTUP_WARMUP`: comma-separated dataset ids (`release/species/assembly`)
- `ATLAS_FAIL_ON_WARMUP_ERROR`: fail readiness behavior toggle

Recommended production profile:

1. Pin high-traffic datasets.
2. Warm those datasets during startup.
3. Enable fail-on-warmup for strict SLO environments.

This reduces cold-start tail latency for first user requests.
