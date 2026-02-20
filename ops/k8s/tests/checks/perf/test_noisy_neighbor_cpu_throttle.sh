#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need curl

wait_ready
with_port_forward 18080
"$ROOT/ops/load/scripts/run_suite.sh" noisy-neighbor-cpu-throttle.json "$ROOT/artifacts/perf/results" >/dev/null
curl -fsS "$BASE_URL/healthz" >/dev/null || {
  echo "failure_mode: noisy_neighbor_healthz_unavailable" >&2
  exit 1
}
echo "noisy neighbor cpu throttle contract passed"
