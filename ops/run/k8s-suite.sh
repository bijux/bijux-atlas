#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-k8s-suite"
PROFILE="${PROFILE:-kind}"
ops_context_guard "$PROFILE"
ops_version_guard kind kubectl helm
SUITE="${SUITE:-full}"
start="$(date +%s)"
log_dir="ops/_generated/k8s-suite/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! ./ops/k8s/tests/suite.sh --suite "$SUITE" "$@" >"$log_file" 2>&1; then
  status="fail"
fi
end="$(date +%s)"
ops_write_lane_report "k8s-suite" "${RUN_ID}" "${status}" "$((end - start))" "${log_file}" "ops/_generated" >/dev/null
[ "$status" = "pass" ] || exit 1
