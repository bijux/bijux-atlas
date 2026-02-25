# Observability Pack

- Owner: `bijux-atlas-operations`

## Contract

- Inputs:
  - `configs/ops/observability-pack.json`
  - `ops/observe/pack/compose/docker-compose.yml`
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
- Pin and verify base toolchain via `configs/ops/tool-versions.json` and `make ops-tools-check`.

Namespace/storage conventions:
- Kubernetes namespace: `atlas-observability`.
- Storage mode default: ephemeral (`ATLAS_OBS_STORAGE_MODE=ephemeral`).
- Optional persistent mode: `ATLAS_OBS_STORAGE_MODE=persistent`.

## Commands

- `make ops-observability-mode ATLAS_OBS_PROFILE=local-compose`
- `make ops-observability-mode ATLAS_OBS_PROFILE=kind`
- `make ops-observability-mode ATLAS_OBS_PROFILE=cluster`
- `make ops-observability-pack-verify`
- `make ops-observability-pack-health`
- `make ops-observability-pack-smoke`
- `make ops-observability-pack-export`
- `make ops-observability-pack-upgrade-check`
- `make ops-observability-pack-conformance-report`
- `make ops-observability-down`

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
