# Ops Observability (Stub)

- Owner: `bijux-atlas-operations`

Canonical documentation lives in `docs/operations/observability/INDEX.md`.

Local source-of-truth assets remain under this directory (`alerts/`, `contract/`, `grafana/`, `scripts/`, `tests/`).
Use make targets for operations:

- `make ops-observability-validate`
- `make ops-observability-pack-tests`
- `make ops-obs-mode ATLAS_OBS_PROFILE=local-compose|kind|cluster`
- `make ops-observability-pack-verify`
- `make ops-observability-pack-health`
- `make ops-observability-pack-export`
- `make ops-observability-pack-conformance-report`
