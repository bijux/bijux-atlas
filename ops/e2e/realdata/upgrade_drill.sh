#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
. "$ROOT/ops/k8s/tests/k8s-suite-lib.sh"
need helm; need kubectl; need curl

"$ROOT/ops/e2e/realdata/run_two_release_diff.sh"

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
Q="/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1"
BEFORE="$(curl -fsS "$BASE_URL$Q")"

errors_file="$(mktemp)"
(
  i=0
  while [ $i -lt 120 ]; do
    if ! curl -fsS "$BASE_URL/healthz" >/dev/null; then
      echo "healthz" >> "$errors_file"
    fi
    if ! curl -fsS "$BASE_URL$Q" >/dev/null; then
      echo "genes" >> "$errors_file"
    fi
    i=$((i+1))
    sleep 0.5
  done
) &
probe_pid=$!

helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" \
  --set server.responseMaxBytes=393216 \
  --set server.requestTimeoutMs=5500 \
  --wait >/dev/null

wait "$probe_pid"

if [ -s "$errors_file" ]; then
  echo "upgrade drill had request failures:" >&2
  cat "$errors_file" >&2
  exit 1
fi

AFTER="$(curl -fsS "$BASE_URL$Q")"
python3 - <<'PY' "$BEFORE" "$AFTER"
import json,sys
b=json.loads(sys.argv[1]); a=json.loads(sys.argv[2])
assert b.get("rows") == a.get("rows"), "semantic drift after upgrade"
PY

echo "upgrade drill passed"
