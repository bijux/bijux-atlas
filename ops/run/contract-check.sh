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

start="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
status="pass"
failure=""

run_step() {
  local name="$1"
  shift
  if ! "$@"; then
    status="fail"
    failure="$name"
    return 1
  fi
}

run_step generate-layer-contract python3 "$ROOT/ops/_meta/generate_layer_contract.py"
run_step check-layer-contract-drift python3 "$ROOT/ops/_lint/check_layer_contract_drift.py"
run_step validate-ops-contracts python3 "$ROOT/scripts/layout/validate_ops_contracts.py"
run_step check-literals python3 "$ROOT/ops/_lint/no-layer-literals.py"
run_step check-k8s-layer-contract "$ROOT/ops/k8s/tests/checks/obs/test_layer_contract_render.sh"
run_step check-live-layer-contract "$ROOT/ops/stack/tests/validate_live_snapshot.sh" "$OUT_DIR/live-snapshot.json"

end="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
python3 - "$OUT_JSON" "$RUN_ID" "$status" "$failure" "$start" "$end" <<'PY'
import json
import sys

out, run_id, status, failure, start, end = sys.argv[1:7]
obj = {
    "run_id": run_id,
    "status": status,
    "failure": failure,
    "start": start,
    "end": end,
    "contract": "ops/_meta/layer-contract.json",
    "checks": [
        "generate-layer-contract",
        "check-layer-contract-drift",
        "validate-ops-contracts",
        "check-literals",
        "check-k8s-layer-contract",
        "check-live-layer-contract",
    ],
}
with open(out, "w", encoding="utf-8") as f:
    json.dump(obj, f, indent=2, sort_keys=True)
    f.write("\n")
PY

cat "$OUT_JSON"
[ "$status" = "pass" ]
