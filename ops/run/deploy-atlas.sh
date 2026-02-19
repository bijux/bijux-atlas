#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="${RUN_ID:-${OPS_RUN_ID:-}}"
export ARTIFACT_DIR="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
ops_entrypoint_start "ops-deploy"
exec ./ops/e2e/scripts/deploy_atlas.sh "$@"
