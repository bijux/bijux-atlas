#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SUITE="${1:?suite file required, e.g. mixed_80_20.js}"
OUT_DIR="${2:-$ROOT/artifacts/perf/results}"
BASE_URL="${BASE_URL:-http://127.0.0.1:18080}"

mkdir -p "$OUT_DIR"
SUMMARY_JSON="$OUT_DIR/${SUITE%.js}.summary.json"

if command -v k6 >/dev/null 2>&1; then
  BASE_URL="$BASE_URL" k6 run --summary-export "$SUMMARY_JSON" "$ROOT/ops/load/k6/suites/$SUITE"
else
  docker run --rm --network host \
    -e BASE_URL="$BASE_URL" \
    -v "$ROOT:/work" -w /work \
    grafana/k6:0.49.0 run --summary-export "$SUMMARY_JSON" "ops/load/k6/suites/$SUITE"
fi

echo "suite complete: $SUITE -> $SUMMARY_JSON"