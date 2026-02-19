#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-obs-verify"
ops_version_guard kind kubectl helm python3
SUITE="${SUITE:-full}"
if [ "${1:-}" = "--suite" ]; then
  SUITE="${2:-full}"
  shift 2
fi
start="$(date +%s)"
log_dir="ops/_generated/obs-verify/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! ./ops/obs/tests/suite.sh --suite "$SUITE" "$@" >"$log_file" 2>&1; then
  status="fail"
fi
end="$(date +%s)"
ops_write_lane_report "obs-verify" "${RUN_ID}" "${status}" "$((end - start))" "${log_file}" "ops/_generated" >/dev/null
[ "$status" = "pass" ] || exit 1
