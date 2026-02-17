#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need kubectl; need curl

install_chart
wait_ready
with_port_forward 18080
trap 'stop_port_forward' EXIT

# Service endpoint reachable
curl -fsS "$BASE_URL/healthz" >/dev/null
# Readiness semantics: ready endpoint must answer success after rollout
curl -fsS "$BASE_URL/readyz" >/dev/null

echo "install gate passed"
