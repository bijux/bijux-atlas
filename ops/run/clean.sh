#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-clean"
days="${OPS_RETENTION_DAYS:-7}"
find artifacts/ops -mindepth 1 -maxdepth 1 -type d -mtime +"$days" -exec rm -rf {} + 2>/dev/null || true
rm -rf artifacts/perf/results artifacts/e2e-datasets artifacts/e2e-store
