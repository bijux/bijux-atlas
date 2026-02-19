#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-load-suite"
PROFILE="${PROFILE:-kind}"
ops_context_guard "$PROFILE"
ops_version_guard k6
SUITE="${SUITE:-mixed-80-20}"
OUT="${OUT:-artifacts/perf/results}"
if [ "$SUITE" = "mixed-80-20" ]; then
  SUITE="mixed"
fi
start="$(date +%s)"
log_dir="ops/_generated/load-suite/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! ./ops/load/scripts/run_suite.sh "${SUITE}.json" "$OUT" >"$log_file" 2>&1; then
  status="fail"
fi
end="$(date +%s)"
ops_write_lane_report "load-suite" "${RUN_ID}" "${status}" "$((end - start))" "${log_file}" "ops/_generated" >/dev/null
[ "$status" = "pass" ] || exit 1
