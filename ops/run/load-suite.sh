#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_require_run_context
SUITE="${SUITE:-mixed-80-20}"
OUT="${OUT:-artifacts/perf/results}"
if [ "$SUITE" = "mixed-80-20" ]; then
  SUITE="mixed"
fi
exec ./ops/load/scripts/run_suite.sh "${SUITE}.json" "$OUT"
