#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm grep
rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" >"$rendered"
grep -q "kind: HorizontalPodAutoscaler" "$rendered"
grep -q "behavior:" "$rendered"
grep -q "scaleUp:" "$rendered"
grep -q "scaleDown:" "$rendered"
grep -q "policies:" "$rendered"
echo "hpa behavior contract passed"
        """,
        Path(__file__),
    )

if __name__ == "__main__":
    raise SystemExit(main())
