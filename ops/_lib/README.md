# Ops Shell Library

Shared shell helpers for ops scripts and tests.

Rules:
- Keep reusable logic in `ops/_lib/`.
- Ops scripts should source `ops/_lib/common.sh` for path resolution and shared wrappers.
- K8s tests may source `ops/e2e/k8s/tests/common.sh` for compatibility, which delegates to `_lib`.
- Shell linting is required through `ops/_lib/shellcheck.sh`.

Files:
- `common.sh`: shared retries, timeout wrappers, kubectl wait wrappers, artifact capture helpers.
- `k8s-test-common.sh`: k8s test helpers built on `common.sh`.
- `k8s-test-report.sh`: failure report collector built on `common.sh`.
- `shellcheck.sh`: shell lint wrapper for all ops scripts.

Stable function contract (`common.sh`):
- `ops_need_cmd <cmd>`
- `ops_mkdir_artifacts`
- `ops_retry <attempts> <sleep_seconds> <cmd ...>`
- `ops_timeout_run <seconds> <cmd ...>`
- `ops_kubectl_wait_condition <ns> <kind> <name> <condition> [timeout]`
- `ops_capture_artifacts <ns> <release> <out_dir>`

Compatibility notes:
- `ops/_lib/` is the canonical shared ops shell library.
- New reusable ops shell helpers must be added in `ops/_lib/`, not duplicated in other `ops/**` paths.
