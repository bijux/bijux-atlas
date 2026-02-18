#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-k8s-suite"
PROFILE="${PROFILE:-kind}"
ops_context_guard "$PROFILE"
ops_version_guard kind kubectl helm
exec ./ops/k8s/tests/run_all.sh "$@"
