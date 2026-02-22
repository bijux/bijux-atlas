#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need curl

install_chart
wait_ready
with_port_forward 18080
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" prom-outage

echo "prom outage drill passed"
