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
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" store-latency-injection

echo "toxiproxy latency drill test passed"
