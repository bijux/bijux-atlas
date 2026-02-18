#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
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
python3 "$ROOT/ops/observability/scripts/validate_logs_schema.py" --namespace "$NS" --release "$RELEASE" --strict-live

echo "k8s log json gate passed"
