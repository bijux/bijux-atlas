#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need curl

: "${ATLAS_E2E_ENABLE_OTEL:=0}"
if [ "$ATLAS_E2E_ENABLE_OTEL" != "1" ]; then
  echo "otel disabled; skip"
  exit 0
fi

install_chart
wait_ready
with_port_forward 18080
"$ROOT/ops/obs/scripts/run_drill.sh" otel-outage

echo "otel outage drill passed"
