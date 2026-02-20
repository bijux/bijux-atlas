#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-artifacts-open"
ops_version_guard python3
latest="$(find artifacts/ops -mindepth 1 -maxdepth 1 -type d -print 2>/dev/null | sort | tail -n1 || true)"
[ -n "$latest" ] || ops_fail "$OPS_ERR_ARTIFACT" "no artifacts found under artifacts/ops"
echo "$latest"
if command -v open >/dev/null 2>&1; then open "$latest" >/dev/null 2>&1 || true; fi
if command -v xdg-open >/dev/null 2>&1; then xdg-open "$latest" >/dev/null 2>&1 || true; fi
