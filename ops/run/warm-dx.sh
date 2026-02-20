#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"

ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-warm-dx"
ops_version_guard python3 kind kubectl

./ops/run/warm-entrypoint.sh --mode datasets
./ops/run/warm-entrypoint.sh --mode shards
./ops/run/cache-status.sh

out_dir="ops/_evidence/warm/${RUN_ID}"
mkdir -p "$out_dir"
cp -f artifacts/ops/cache-status/report.json "$out_dir/cache-status.json" 2>/dev/null || true
printf '%s\n' "warm completed run_id=${RUN_ID}" > "$out_dir/summary.txt"
echo "$out_dir"
