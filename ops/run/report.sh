#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-report-merge"
ops_version_guard python3

run_id="${RUN_ID}"
out="ops/_generated/report.unified.json"
schema="ops/_schemas/report/unified.schema.json"
export OPS_REPORT_RUN_ID="$run_id"
export OPS_REPORT_OUT="$out"
export OPS_REPORT_SCHEMA="$schema"

python3 - <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

root = Path(".")
run_id = os.environ["OPS_REPORT_RUN_ID"]
out = root / os.environ["OPS_REPORT_OUT"]
schema_path = root / os.environ["OPS_REPORT_SCHEMA"]

lanes = {}
generated_root = root / "ops/_generated"
for lane_dir in sorted(p for p in generated_root.iterdir() if p.is_dir()):
    lane = lane_dir.name
    candidate = lane_dir / run_id / "report.json"
    legacy = lane_dir / f"{run_id}.json"
    f = candidate if candidate.exists() else legacy
    if f.exists():
        lanes[lane] = json.loads(f.read_text(encoding="utf-8"))

summary = {
    "total": len(lanes),
    "passed": sum(1 for v in lanes.values() if v.get("status") == "pass"),
    "failed": sum(1 for v in lanes.values() if v.get("status") == "fail"),
}

payload = {
    "schema_version": 1,
    "run_id": run_id,
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "lanes": lanes,
    "summary": summary,
}

schema = json.loads(schema_path.read_text(encoding='utf-8'))
for key in schema.get("required", []):
    if key not in payload:
        raise SystemExit(f"missing required unified-report field: {key}")

lane_schema = schema.get("properties", {}).get("lanes", {}).get("additionalProperties", {})
for lane_name, lane_payload in lanes.items():
    for key in lane_schema.get("required", []):
        if key not in lane_payload:
            raise SystemExit(f"lane `{lane_name}` missing required field `{key}`")
    if lane_payload.get("run_id") != run_id:
        raise SystemExit(f"lane `{lane_name}` run_id mismatch: {lane_payload.get('run_id')} != {run_id}")

out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding='utf-8')
print(out)
PY
