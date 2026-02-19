#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl curl

if ! kubectl api-resources | grep -q "^servicemonitors"; then
  echo "ServiceMonitor CRD not present; skipping scrape gate in local cluster"
  exit 0
fi

install_chart --set serviceMonitor.enabled=true
wait_ready
kubectl -n "$NS" get servicemonitor "$SERVICE_NAME" >/dev/null
with_port_forward 18080
wait_for_http "$BASE_URL/metrics" 200 60
curl -fsS "$BASE_URL/metrics" | grep -q 'bijux_'

echo "service monitor gate passed"
