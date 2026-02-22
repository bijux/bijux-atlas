#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set nodeLocalSsdProfile.enabled=true | grep -q 'emptyDir:'
echo "node-local cache profile gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
