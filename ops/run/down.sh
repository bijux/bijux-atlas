#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"

ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-down"
ops_version_guard kind kubectl helm

cluster="${ATLAS_E2E_CLUSTER_NAME:?ATLAS_E2E_CLUSTER_NAME is required by configs/ops/env.schema.json}"
if ! kind get clusters 2>/dev/null | grep -qx "$cluster"; then
  echo "ops-down: kind cluster not present; nothing to do"
  exit 0
fi

exec ./ops/run/stack-down.sh
