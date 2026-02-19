#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-e2e-smoke"
ops_version_guard kubectl python3
start="$(date +%s)"
log_dir="ops/_generated/e2e-smoke/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! { ./ops/e2e/scripts/smoke_queries.sh && python3 ./ops/e2e/smoke/generate_report.py; } >"$log_file" 2>&1; then
  {
    echo "ops-e2e-smoke: initial smoke failed; bootstrapping stack+deploy and retrying once"
    make -s ops-up
    make -s ops-deploy
    ./ops/e2e/scripts/smoke_queries.sh
    python3 ./ops/e2e/smoke/generate_report.py
  } >>"$log_file" 2>&1 || status="fail"
fi
end="$(date +%s)"
ops_write_lane_report "e2e-smoke" "${RUN_ID}" "${status}" "$((end - start))" "${log_file}" "ops/_generated" >/dev/null
[ "$status" = "pass" ] || exit 1
