#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps

install_chart
wait_ready
ATLAS_BASE_URL="$BASE_URL" "$ROOT/ops/load/scripts/run_suite.sh" noisy-neighbor-cpu-throttle.json artifacts/perf/results

# keep permissive: scenario completion is the signal
[ -f artifacts/perf/results/noisy-neighbor-cpu-throttle.json ] || {
  echo "expected noisy-neighbor result missing" >&2
  exit 1
}

echo "cpu throttle noisy-neighbor scenario passed"
