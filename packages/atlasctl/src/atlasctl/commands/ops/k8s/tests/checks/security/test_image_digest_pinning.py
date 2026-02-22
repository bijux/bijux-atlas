#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm; need grep
PROFILE="$ROOT/ops/k8s/values/perf.yaml"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/perf-pin.yaml
grep -Eq 'image: ".+@sha256:[a-f0-9]{64}"' /tmp/perf-pin.yaml
echo "image digest pinning gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
