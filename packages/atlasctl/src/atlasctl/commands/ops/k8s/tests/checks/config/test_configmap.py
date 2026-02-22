#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm; need kubectl
install_chart --set concurrency.cheap=not-a-number
sleep 8
POD="$(pod_name || true)"
[ -n "$POD" ] || { echo "no pod created for configmap test" >&2; exit 1; }
STATE="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.status.containerStatuses[0].state.waiting.reason}' || true)"
[ "$STATE" = "CrashLoopBackOff" ] || [ "$STATE" = "Error" ] || {
  echo "expected startup failure for invalid configmap value, got: $STATE" >&2
  exit 1
}
echo "configmap gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
