#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
requested_run_id="${RUN_ID:-}"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="${ARTIFACT_DIR:-$OPS_RUN_DIR}"
ops_env_load
ops_entrypoint_start "ops-evidence-open"
ops_version_guard python3
area="${AREA:-make}"
run_id="${requested_run_id}"
if [ -z "$run_id" ] && [ -f "artifacts/evidence/${area}/latest-run-id.txt" ]; then
  run_id="$(cat "artifacts/evidence/${area}/latest-run-id.txt")"
fi
if [ -z "$run_id" ] && [ -f "artifacts/evidence/latest-run-id.txt" ]; then
  run_id="$(cat artifacts/evidence/latest-run-id.txt)"
fi
[ -n "$run_id" ] || ops_fail "$OPS_ERR_ARTIFACT" "no run id found (set RUN_ID or AREA and ensure latest-run-id.txt exists)"
target="artifacts/evidence/${area}/${run_id}"
[ -d "$target" ] || ops_fail "$OPS_ERR_ARTIFACT" "missing evidence directory: $target"
echo "$target"
if command -v open >/dev/null 2>&1; then open "$target" >/dev/null 2>&1 || true; fi
if command -v xdg-open >/dev/null 2>&1; then xdg-open "$target" >/dev/null 2>&1 || true; fi
