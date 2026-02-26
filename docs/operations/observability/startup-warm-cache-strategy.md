# Startup Warm-Cache Strategy

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Atlas supports startup warm-cache through environment configuration:

- `ATLAS_STARTUP_WARMUP`: comma-separated dataset ids (`release/species/assembly`)
- `ATLAS_FAIL_ON_WARMUP_ERROR`: fail readiness behavior toggle

## Install

- Pin high-traffic datasets.
- Configure startup warm targets.

## Verify

- Confirm warm targets are fetched during startup.
- Confirm readiness behavior for warmup failures matches policy.

## Run drills

- Simulate store latency during startup.
- Verify warmup retries and readiness behavior.

This reduces cold-start tail latency for first user requests.

## See also

- `ops-ci`
