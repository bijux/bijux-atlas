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
