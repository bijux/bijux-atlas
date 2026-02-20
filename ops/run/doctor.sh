#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-doctor"
ops_version_guard kind kubectl helm k6
./ops/run/prereqs.sh
echo "evidence root: artifacts/evidence"
echo "evidence run id pointer: artifacts/evidence/latest-run-id.txt"
python3 ./scripts/areas/layout/check_tool_versions.py kind kubectl helm k6 jq yq python3 || true
python3 ./scripts/areas/layout/check_ops_pins.py || true
if rg -n "(?:legacy/[A-Za-z0-9_.-]+|ops-[A-Za-z0-9-]+-legacy|ops/.*/_legacy/|ops/.*/scripts/.*legacy)" \
  makefiles docs .github/workflows >/dev/null 2>&1; then
  echo "legacy ops path/target references found in public surfaces" >&2
  rg -n "(?:legacy/[A-Za-z0-9_.-]+|ops-[A-Za-z0-9-]+-legacy|ops/.*/_legacy/|ops/.*/scripts/.*legacy)" \
    makefiles docs .github/workflows || true
  exit 1
fi
pin_report="artifacts/evidence/pins/${RUN_ID}/pin-drift-report.json"
if [ -f "$pin_report" ]; then
  echo "pin drift report: $pin_report"
  cat "$pin_report"
fi
make -s ops-env-print || true
