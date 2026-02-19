# Ops E2E

## Purpose
Own composition-only end-to-end scenarios across stack, k8s, obs, load, and datasets.

## Entry points
- `./ops/run/e2e.sh --suite smoke`
- `./ops/run/e2e.sh --suite k8s-suite --profile kind`
- `./ops/run/e2e.sh --suite realdata --fast`
- `./ops/run/e2e.sh --suite smoke --no-deploy`
- `make ops-e2e SUITE=smoke|k8s-suite|realdata`
- `make ops-local-full`
- `make ops-e2e-validate`

## Contracts
- `ops/e2e/CONTRACT.md`
- `ops/e2e/suites/suites.json`

## Artifacts
- `ops/_artifacts/<run_id>/e2e/`

## Failure modes
- Scenario manifest references missing subarea assets.
- Composed workflow fails under outage/restart drills.
- API smoke snapshot drift.
