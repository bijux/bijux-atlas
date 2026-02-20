#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-obs-verify"
ops_version_guard kind kubectl helm python3
SUITE="${SUITE:-full}"
if [ "${1:-}" = "--suite" ]; then
  SUITE="${2:-full}"
  shift 2
fi
start="$(date +%s)"
log_dir="artifacts/evidence/obs-verify/${RUN_ID}"
mkdir -p "$log_dir"
log_file="$log_dir/run.log"
status="pass"
if ! ./ops/obs/tests/suite.sh --suite "$SUITE" "$@" >"$log_file" 2>&1; then
  status="fail"
fi
end="$(date +%s)"
python3 ./ops/obs/scripts/areas/contracts/write_obs_conformance_report.py \
  --run-id "${RUN_ID}" \
  --suite "${SUITE}" \
  --status "${status}" \
  --out-dir "${log_dir}" >/dev/null || true
cp -f artifacts/ops/obs/traces.exemplars.log "${log_dir}/traces.exemplars.log" 2>/dev/null || true
if [ "${status}" = "pass" ]; then
  mkdir -p artifacts/evidence/obs
  python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("artifacts/evidence/obs/last-pass.json")
payload = {
  "run_id": "${RUN_ID}",
  "suite": "${SUITE}",
  "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
fi
ops_write_lane_report "obs-verify" "${RUN_ID}" "${status}" "$((end - start))" "${log_file}" "artifacts/evidence" >/dev/null
[ "$status" = "pass" ] || exit 1
