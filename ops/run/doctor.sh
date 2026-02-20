#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-doctor"
ops_version_guard kind kubectl helm k6
./ops/run/prereqs.sh
echo "evidence root: artifacts/evidence"
echo "evidence run id pointer: artifacts/evidence/latest-run-id.txt"
python3 ./scripts/layout/check_tool_versions.py kind kubectl helm k6 jq yq python3 || true
python3 ./scripts/layout/check_ops_pins.py || true
pin_report="artifacts/evidence/pins/${RUN_ID}/pin-drift-report.json"
if [ -f "$pin_report" ]; then
  echo "pin drift report: $pin_report"
  cat "$pin_report"
fi
make -s ops-env-print || true
