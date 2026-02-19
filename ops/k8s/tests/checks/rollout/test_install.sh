#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm; need kubectl; need curl

install_chart
wait_ready
with_port_forward 18080

# Service endpoint reachable
wait_for_http "$BASE_URL/healthz" 200 60
echo "install gate passed"
