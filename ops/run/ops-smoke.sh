#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="${RUN_ID:-${OPS_RUN_ID:-}}"
export ARTIFACT_DIR="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
ops_entrypoint_start "ops-smoke"

out_dir="ops/_generated/ops-smoke"
mkdir -p "$out_dir"
out_file="$out_dir/${RUN_ID}.json"

status="pass"
if ! make -s ops-smoke-legacy; then
  status="fail"
fi

python3 - <<PY
import json
from datetime import datetime, timezone
payload = {
  "schema_version": 1,
  "lane": "ops-smoke",
  "run_id": "${RUN_ID}",
  "status": "${status}",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "artifacts": {"ops_generated": "${out_file}"},
}
with open("${out_file}", "w", encoding="utf-8") as f:
  json.dump(payload, f, indent=2, sort_keys=True)
  f.write("\\n")
print("${out_file}")
PY

./ops/run/report.sh >/dev/null
[ "$status" = "pass" ] || exit 1
