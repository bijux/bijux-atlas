# Ops Stack

## Purpose
Own local and CI infrastructure dependencies: kind, store backend, redis, telemetry stack, and fault primitives.

## Entry points
- `make ops-stack-up PROFILE=kind`
- `make ops-stack-down`
- `make ops-stack-smoke`
- `make ops-stack-health-report`

## Contracts
- `ops/stack/CONTRACT.md`
- `ops/inventory/toolchain.yaml`

## Artifacts
- `ops/_artifacts/<run_id>/stack/`

## Failure modes
- Cluster bootstrap fails due to docker/kind toolchain drift.
- Dependency readiness timeout (store/otel/prom/grafana/redis).
- Context mismatch prevented by guardrails.
