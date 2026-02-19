#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-doctor"
ops_version_guard kind kubectl helm k6
./ops/run/prereqs.sh
python3 ./scripts/layout/check_tool_versions.py kind kubectl helm k6 jq yq python3 || true
python3 ./scripts/layout/check_ops_pins.py || true
if [ -f ops/_generated/pins/pin-drift-report.json ]; then
  echo "pin drift report: ops/_generated/pins/pin-drift-report.json"
  cat ops/_generated/pins/pin-drift-report.json
fi
make -s ops-env-print || true
