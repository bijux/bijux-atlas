#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl python3

install_chart
wait_ready
sleep 3
LOG_LINE="$(kubectl -n "$NS" logs deploy/"$SERVICE_NAME" --tail=200 | grep -m1 '{' || true)"
[ -n "$LOG_LINE" ] || { echo "no json-like log line found" >&2; exit 1; }
python3 - <<'PY' "$LOG_LINE"
import json,sys
line=sys.argv[1]
obj=json.loads(line)
assert isinstance(obj,dict)
PY
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py" --namespace "$NS" --release "$RELEASE"

echo "k8s log json gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
