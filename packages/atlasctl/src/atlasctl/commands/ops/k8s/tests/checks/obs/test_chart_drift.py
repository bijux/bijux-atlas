#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell("""
setup_test_traps
need helm shasum
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > /tmp/chart-a.yaml
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > /tmp/chart-b.yaml
A="$(shasum -a 256 /tmp/chart-a.yaml | awk '{print $1}')"
B="$(shasum -a 256 /tmp/chart-b.yaml | awk '{print $1}')"
[ "$A" = "$B" ] || { echo "chart drift detected across two renders" >&2; exit 1; }
echo "chart drift gate passed"
    """, Path(__file__))

if __name__ == "__main__":
    raise SystemExit(main())
