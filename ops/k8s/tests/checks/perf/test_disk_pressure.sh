#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl; need curl

install_chart
wait_ready
with_port_forward 18080
pod="$(pod_name)"
before_ready="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.readyReplicas}' || echo 0)"
# Best-effort disk pressure simulation inside writable tmpdir
kubectl -n "$NS" exec "$pod" -- sh -ceu 'dd if=/dev/zero of=/tmp/atlas-pressure.bin bs=1M count=64 >/dev/null 2>&1 || true'
curl -fsS "$BASE_URL/healthz" >/dev/null || { echo "failure_mode: disk_pressure_healthz_unavailable" >&2; exit 1; }
after_ready="$(kubectl -n "$NS" get deploy "$SERVICE_NAME" -o jsonpath='{.status.readyReplicas}' || echo 0)"
if [ "${after_ready:-0}" -lt 1 ] || [ "${after_ready:-0}" -lt "${before_ready:-1}" ]; then
  echo "failure_mode: disk_pressure_readiness_regressed" >&2
  exit 1
fi
kubectl -n "$NS" exec "$pod" -- rm -f /tmp/atlas-pressure.bin >/dev/null 2>&1 || true

echo "disk pressure readiness contract passed"
