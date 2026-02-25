# Ops Stack

## Purpose
Own local and CI infrastructure dependencies: kind, store backend, redis, telemetry stack, and fault primitives.

## Entry points
- `bijux dev atlas ops stack plan --profile kind`
- `bijux dev atlas ops stack up --profile kind --allow-subprocess --allow-write --allow-network`
- `bijux dev atlas ops stack down --profile kind --allow-subprocess --allow-write --allow-network`
- `bijux dev atlas ops stack status --profile kind --allow-subprocess --format json`
- `bijux dev atlas ops stack reset --reset-run-id <run_id>`

Stack lifecycle is controlled only through `bijux dev atlas ops stack ...`.

## Contracts
- `ops/stack/CONTRACT.md`
- `ops/inventory/pins.yaml` (canonical image pin source)
- `ops/inventory/toolchain.json`
- `ops/stack/generated/version-manifest.json` (derived mirror)

## Artifacts
- `ops/_artifacts/<run_id>/stack/`
- `ops/stack/generated/stack-index.json`
- `ops/stack/generated/dependency-graph.json`
- `ops/stack/generated/artifact-metadata.json`

## Failure modes
- Cluster bootstrap fails due to docker/kind toolchain drift.
- Dependency readiness timeout (store/otel/prom/grafana/redis).
- Context mismatch prevented by guardrails.
