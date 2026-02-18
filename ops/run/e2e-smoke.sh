#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_require_run_context
./ops/e2e/scripts/smoke_queries.sh
python3 ./ops/e2e/smoke/generate_report.py
