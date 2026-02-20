#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-obs-validate"
ops_version_guard kind kubectl helm python3
exec make ops-observability-validate
