#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-obs-verify"
SUITE="${SUITE:-full}"
if [ "${1:-}" = "--suite" ]; then
  SUITE="${2:-full}"
  shift 2
fi
exec ./ops/obs/tests/suite.sh --suite "$SUITE" "$@"
