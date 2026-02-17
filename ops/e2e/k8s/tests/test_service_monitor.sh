#!/usr/bin/env bash
set -euo pipefailo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl curl

install_chart
wait_ready
kubectl -n "$NS" get servicemonitor "$SERVICE_NAME" >/dev/null
with_port_forward 18080
wait_for_http "$BASE_URL/metrics" 200 60
curl -fsS "$BASE_URL/metrics" | grep -q 'bijux_'

echo "service monitor gate passed"
