#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"

ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-warm"
ops_version_guard kind kubectl

profile="${PROFILE:-kind}"
if ! ops_context_guard "$profile"; then
  if [ "$profile" = "kind" ]; then
    echo "ops-warm: kind context missing; bootstrapping stack" >&2
    make -s ops-stack-up PROFILE=kind
  fi
fi
ops_context_guard "$profile"

exec ./ops/e2e/scripts/warmup.sh "$@"
