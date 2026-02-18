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
- `shellcheck.sh`: shell lint wrapper using `configs/shellcheck/shellcheckrc`.

Policy:
- Do not copy these helpers into other locations.
- New shared ops shell helpers must be added here and documented.
- Scripts under `ops/**/scripts` may source only from `ops/_lib/*` for shared logic.
