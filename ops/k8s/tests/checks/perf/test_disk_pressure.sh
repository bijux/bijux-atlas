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
# Best-effort disk pressure simulation inside writable tmpdir
kubectl -n "$NS" exec "$pod" -- sh -ceu 'dd if=/dev/zero of=/tmp/atlas-pressure.bin bs=1M count=64 >/dev/null 2>&1 || true'
curl -fsS "$BASE_URL/healthz" >/dev/null
kubectl -n "$NS" exec "$pod" -- rm -f /tmp/atlas-pressure.bin >/dev/null 2>&1 || true

echo "disk pressure simulation passed"
