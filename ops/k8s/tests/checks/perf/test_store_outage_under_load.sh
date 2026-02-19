#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need curl

install_chart
wait_ready
with_port_forward 18080
"$ROOT/ops/obs/scripts/run_drill.sh" store-outage-under-load

curl -fsS "$BASE_URL/healthz" >/dev/null

echo "minio outage mid-load drill passed"
