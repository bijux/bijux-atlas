#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-evidence-open"
ops_version_guard python3
run_id="${RUN_ID:-$(cat ops/_evidence/latest-run-id.txt 2>/dev/null || true)}"
[ -n "$run_id" ] || ops_fail "$OPS_ERR_ARTIFACT" "no run id found in ops/_evidence/latest-run-id.txt"
latest="ops/_evidence/make/$run_id"
[ -d "$latest" ] || ops_fail "$OPS_ERR_ARTIFACT" "missing evidence directory: $latest"
echo "$latest"
if command -v open >/dev/null 2>&1; then open "$latest" >/dev/null 2>&1 || true; fi
if command -v xdg-open >/dev/null 2>&1; then xdg-open "$latest" >/dev/null 2>&1 || true; fi
