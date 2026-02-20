#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-contract-check"
ops_version_guard python3 helm kubectl
RUN_ID="${RUN_ID:-$OPS_RUN_ID}"
OUT_DIR="${OPS_RUN_DIR}/contracts"
OUT_JSON="$OUT_DIR/report.json"
mkdir -p "$OUT_DIR"

python3 "$ROOT/ops/run/contract-report.py" --run-id "$RUN_ID" --out "${OUT_JSON#$ROOT/}"
cat "$OUT_JSON"
python3 - "$OUT_JSON" <<'PY'
import json, sys
obj = json.load(open(sys.argv[1], encoding="utf-8"))
raise SystemExit(0 if obj.get("status") == "pass" else 1)
PY
