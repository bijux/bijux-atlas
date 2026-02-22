#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need curl

wait_ready
with_port_forward 18080
(cd "$ROOT" && ./bin/atlasctl ops load --report text run --suite noisy-neighbor-cpu-throttle.json --out artifacts/perf/results) >/dev/null
curl -fsS "$BASE_URL/healthz" >/dev/null || {
  echo "failure_mode: noisy_neighbor_healthz_unavailable" >&2
  exit 1
}
echo "noisy neighbor cpu throttle contract passed"
