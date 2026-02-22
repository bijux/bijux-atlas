#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell("""
setup_test_traps
need helm
for values in "$ROOT"/ops/k8s/values/*.yaml; do
  helm lint "$CHART" -f "$values" >/dev/null
  helm template "$RELEASE" "$CHART" -n "$NS" -f "$values" >/dev/null
done
echo "values profiles validity contract passed"
    """, Path(__file__))

if __name__ == "__main__":
    raise SystemExit(main())
