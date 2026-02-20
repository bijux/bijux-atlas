#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-smoke"
ops_version_guard kind kubectl helm k6

start="$(date +%s)"
log_dir="ops/_generated/ops-smoke/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"

status="pass"
if ! make -s ops-smoke-legacy >"$log_file" 2>&1; then
  status="fail"
fi

end="$(date +%s)"
duration="$((end - start))"
LANE_REPRO_COMMAND="make ops/smoke REUSE=1" \
ops_write_lane_report "ops-smoke" "${RUN_ID}" "${status}" "${duration}" "${log_file}" "ops/_generated" >/dev/null

./ops/run/report.sh >/dev/null
if [ "$status" = "pass" ]; then
  RUN_ID="${RUN_ID}" python3 ./ops/_lint/ops-smoke-budget-check.py
fi
[ "$status" = "pass" ] || exit 1
