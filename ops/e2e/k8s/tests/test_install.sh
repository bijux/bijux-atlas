#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm; need kubectl; need curl

install_chart
wait_ready
with_port_forward 18080

# Service endpoint reachable
wait_for_http "$BASE_URL/healthz" 200 60
# Readiness semantics: ready endpoint must answer success after rollout
wait_for_http "$BASE_URL/readyz" 200 60

echo "install gate passed"
