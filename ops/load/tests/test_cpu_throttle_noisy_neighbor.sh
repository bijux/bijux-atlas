#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps

install_chart
wait_ready
out="${OPS_RUN_DIR:-ops/_artifacts/load/results}"
(
  cd "$ROOT"
  ATLAS_BASE_URL="$BASE_URL" ./bin/atlasctl ops load --report text run --suite noisy-neighbor-cpu-throttle.json --out "$out"
)

[ -f "$out/noisy-neighbor-cpu-throttle.json" ] || {
  echo "expected noisy-neighbor result missing" >&2
  exit 1
}

echo "cpu throttle noisy-neighbor scenario passed"
