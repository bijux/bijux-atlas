#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
SUITE_OR_SCENARIO="${1:?scenario or suite required}"
OUT_DIR="${2:-$ROOT/artifacts/perf/results}"
export ATLAS_BASE_URL="${ATLAS_BASE_URL:-${BASE_URL:-http://127.0.0.1:18080}}"
export ATLAS_API_KEY="${ATLAS_API_KEY:-}"
"$ROOT/scripts/public/perf/check_pinned_queries_lock.py"
"$ROOT/scripts/public/perf/run_suite.sh" "$SUITE_OR_SCENARIO" "$OUT_DIR"
