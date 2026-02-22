#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm; need kubectl
TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<'YAML'
rollout:
  enabled: true
YAML
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES" | grep -q "kind: Rollout"
RENDERED="$(helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$TMP_VALUES")"
printf '%s' "$RENDERED" | grep -q "setWeight: 10"
printf '%s' "$RENDERED" | grep -q "setWeight: 50"
if printf '%s' "$RENDERED" | grep -q "analysis:"; then
  printf '%s' "$RENDERED" | grep -q "templates:"
fi
echo "rollout gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
