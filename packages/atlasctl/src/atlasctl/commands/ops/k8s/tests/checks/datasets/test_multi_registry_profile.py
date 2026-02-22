#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm shasum
PROFILE="$ROOT/ops/k8s/values/multi-registry.yaml"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/multi-registry-a.yaml
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/multi-registry-b.yaml
shasum -a 256 /tmp/multi-registry-a.yaml /tmp/multi-registry-b.yaml | awk '{print $1}' | uniq | wc -l | grep -q '^1$'
echo "multi-registry deterministic render gate passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
