# Ops Shell Library

Shared shell helpers for ops scripts and tests.

Rules:
- Keep reusable logic in `ops/_lib/`.
- Ops tests/scripts should source wrappers under `ops/e2e/k8s/tests/common.sh` (compat) or `ops/_lib/*` directly.
- Shell linting is required through `ops/_lib/shellcheck.sh`.

Files:
- `k8s-test-common.sh`: shared k8s test helpers.
- `k8s-test-report.sh`: shared failure report collector.
- `shellcheck.sh`: shell lint wrapper for all ops scripts.
