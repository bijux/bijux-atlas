#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl
install_chart
wait_ready
install_chart
wait_ready
echo "install twice idempotency passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
