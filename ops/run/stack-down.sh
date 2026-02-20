#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-stack-down"
ops_version_guard kind kubectl helm
start_ts="$(date +%s)"
status="pass"
log_file="artifacts/evidence/stack/${RUN_ID}/stack-down.log"
health_json="artifacts/evidence/stack/${RUN_ID}/health-report-after-down.json"
snapshot_json="artifacts/evidence/stack/state-snapshot.json"
atlas_ns="${ATLAS_E2E_NAMESPACE:?ATLAS_E2E_NAMESPACE is required by configs/ops/env.schema.json}"
mkdir -p "$(dirname "$log_file")"
if ! (
  make -s ops-env-validate
  ./ops/stack/kind/context_guard.sh
  ./ops/stack/kind/namespace_guard.sh
  ./ops/stack/scripts/uninstall.sh
) >"$log_file" 2>&1; then
  status="fail"
fi
ATLAS_HEALTH_REPORT_FORMAT=json ./ops/stack/scripts/health_report.sh "$atlas_ns" "$health_json" >/dev/null || true
duration="$(( $(date +%s) - start_ts ))"
ops_write_lane_report "stack" "$RUN_ID" "$status" "$duration" "$log_file"
[ "$status" = "pass" ] && rm -f "$snapshot_json"
[ "$status" = "pass" ] || exit 1
