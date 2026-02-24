# Ops Observability

## Purpose
Own observability pack assets, contracts, drills, and verification routines.

## Entry points
- `make ops-obs-up PROFILE=compose`
- `make ops-obs-verify`
- `bijux dev atlas ops obs verify --suite cheap|contracts|coverage|minimal-drills|root-local|drills|full`
- `make ops-obs-drill DRILL=otel-outage PROFILE=kind`
- `make ops-observability-pack-tests`
- `make obs/update-goldens`

## Contracts
- `ops/obs/CONTRACT.md`
- `configs/ops/observability-pack.json`

## Artifacts
- `ops/_artifacts/<run_id>/obs/`

## Failure modes
- Pack install mismatch with pinned versions.
- Metrics/traces contract violations.
- Drill execution fails to restore healthy state.
