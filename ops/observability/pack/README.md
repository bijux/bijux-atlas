# Observability Pack

- Owner: `bijux-atlas-operations`

## Contract

- Inputs:
  - `configs/ops/observability-pack.json`
  - `ops/observability/pack/compose/docker-compose.yml`
  - `ops/stack/{prometheus,grafana,otel}/*.yaml`
- Outputs:
  - Running observability pack for the selected profile
  - Artifact bundle under `artifacts/observability/pack-bundle/`
- Invariants:
  - Profile must be explicit (`local-compose|kind|cluster`)
  - Versions are pinned and validated by `check_pack_versions.sh`
  - Install/uninstall are idempotent

## Profiles

- `local-compose`: Docker Compose runtime for local reproducibility.
- `kind`: Kubernetes pack in local kind cluster.
- `cluster`: Kubernetes pack with stricter CRD requirements.

Airgapped note:
- Mirror pinned image refs from `configs/ops/observability-pack.json` into your local registry.
- Keep digest fields updated to the mirrored immutable artifacts.

## Commands

- `make ops-obs-mode ATLAS_OBS_PROFILE=local-compose`
- `make ops-obs-mode ATLAS_OBS_PROFILE=kind`
- `make ops-obs-mode ATLAS_OBS_PROFILE=cluster`
- `make ops-observability-pack-verify`
- `make ops-observability-pack-smoke`
- `make ops-observability-pack-export`
- `make ops-obs-down`
