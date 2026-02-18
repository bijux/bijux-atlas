# ops/_lib

Canonical shared shell library for ops workflows.

Stable contracts:
- `run_id.sh`: deterministic run-id/namespace/artifact-dir initialization.
- `common.sh`: retry helpers, timeout wrappers, wrapper imports, artifact capture.
- `artifacts.sh`: canonical artifact paths under `artifacts/ops/<run-id>/...`.
- `retry.sh`: bounded retry wrappers.
- `timeout.sh`: bounded timeout wrappers.
- `trap_bundle.sh`: install failure-bundle traps on ERR.
- `kubectl.sh`: kubectl wrapper with retry/timeout and failure bundle dumps.
- `helm.sh`: helm wrapper with retry and failure-debug bundle capture.
- `k8s-test-common.sh`: helpers for k8s e2e test assertions.
- `ports.sh`: canonical service URL helpers.
- `shellcheck.sh`: shell lint wrapper using `configs/shellcheck/shellcheckrc`.

Policy:
- Do not copy these helpers into other locations.
- New shared ops shell helpers must be added here and documented.
- Scripts under `ops/**/scripts` may source only from `ops/_lib/*` for shared logic.

## Make Target

Use \# Ops Index

Use Make targets only.

## Core

- `make ops-help`
- `make ops-surface`
- `make ops-layout-lint`

## Topology

- `make ops-stack-up`
- `make ops-stack-down`
- `make ops-deploy`
- `make ops-undeploy`

## Domains

- `make ops-k8s-tests`
- `make ops-observability-validate`
- `make ops-load-smoke`
- `make ops-dataset-qc`
- `make ops-realdata`

## Full Flows

- `make ops-ref-grade-local`
- `make ops-full` to find the canonical entrypoint for this area.

Canonical docs: `ops/README.md`, `docs/operations/INDEX.md`.
