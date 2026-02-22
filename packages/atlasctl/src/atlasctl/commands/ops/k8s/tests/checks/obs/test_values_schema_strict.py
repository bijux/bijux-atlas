#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell

def main() -> int:
    return run_k8s_test_shell("""
setup_test_traps
need helm grep
bad_values="$(mktemp)"
cat >"$bad_values" <<'YAML'
nonexistentKeyForStrictSchema: true
YAML
if helm lint "$CHART" -f "$bad_values" >/tmp/values-schema-strict.out 2>&1; then
  echo "values schema strict check failed: unknown key unexpectedly accepted" >&2
  exit 1
fi
grep -Eiq "additional property|unknown field|nonexistentKeyForStrictSchema" /tmp/values-schema-strict.out
echo "values schema strict contract passed"
    """, Path(__file__))

if __name__ == "__main__":
    raise SystemExit(main())
