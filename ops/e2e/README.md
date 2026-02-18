# Ops E2E

## Purpose
Own composition-only end-to-end scenarios across stack, k8s, obs, load, and datasets.

## Entry points
- `make ops-e2e-smoke`
- `make ops-realdata REALDATA_SOURCE=ops/datasets/real-datasets.json`
- `make ops-local-full`
- `make ops-e2e-validate`

## Contracts
- `ops/e2e/CONTRACT.md`
- `ops/e2e/scenarios/manifest.json`

## Artifacts
- `ops/_artifacts/<run_id>/e2e/`

## Failure modes
- Scenario manifest references missing subarea assets.
- Composed workflow fails under outage/restart drills.
- API smoke snapshot drift.
