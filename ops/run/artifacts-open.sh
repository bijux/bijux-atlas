#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_require_run_context
latest="$(ls -1dt artifacts/ops/* 2>/dev/null | head -n1 || true)"
[ -n "$latest" ] || { echo "no artifacts" >&2; exit 2; }
echo "$latest"
if command -v open >/dev/null 2>&1; then open "$latest" >/dev/null 2>&1 || true; fi
if command -v xdg-open >/dev/null 2>&1; then xdg-open "$latest" >/dev/null 2>&1 || true; fi
