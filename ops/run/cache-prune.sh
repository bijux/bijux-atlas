#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"

ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-cache-prune"
ops_version_guard python3

rm -rf artifacts/e2e-store artifacts/e2e-datasets artifacts/ops/cache-status
echo "cache prune completed: artifacts/e2e-store artifacts/e2e-datasets artifacts/ops/cache-status"
