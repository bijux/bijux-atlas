#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

: "${ATLAS_E2E_ENABLE_TOXIPROXY:=0}"
if [ "$ATLAS_E2E_ENABLE_TOXIPROXY" != "1" ]; then
  echo "toxiproxy disabled; skip"
  exit 0
fi

install_chart
wait_ready
"$ROOT/ops/obs/scripts/bin/run_drill.sh" store-latency-injection

echo "toxiproxy latency drill test passed"
