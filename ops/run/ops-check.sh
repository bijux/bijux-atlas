#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-check"
ops_version_guard kind kubectl helm k6

start="$(date +%s)"
log_dir="ops/_generated/ops-check/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"

status="pass"
if ! make -s ops-check-legacy >"$log_file" 2>&1; then
  status="fail"
fi

end="$(date +%s)"
duration="$((end - start))"
ops_write_lane_report "ops-check" "${RUN_ID}" "${status}" "${duration}" "${log_file}" "ops/_generated" >/dev/null

[ "$status" = "pass" ] || exit 1
