#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl; need helm
install_chart
wait_ready
helm -n "$NS" uninstall "$RELEASE" >/dev/null
sleep 3
kubectl -n "$NS" get deploy "$SERVICE_NAME" >/dev/null 2>&1 && {
  echo "deployment still exists after uninstall" >&2
  exit 1
}
install_chart
wait_ready
echo "uninstall reinstall gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
