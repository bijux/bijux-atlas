#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell("""
setup_test_traps
need helm grep
rendered="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" >"$rendered"
if grep -Eq 'checksum/(config|secret|values)' "$rendered"; then
  echo "checksum rollout policy violated: deployment uses checksum annotations; contract requires explicit restart flow" >&2
  exit 1
fi
echo "no checksum rollout contract passed"
    """, Path(__file__))

if __name__ == "__main__":
    raise SystemExit(main())
