#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-doctor"
ops_version_guard python3
./ops/run/prereqs.sh
python3 ./scripts/layout/check_tool_versions.py || true
make -s ops-env-print || true
