#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

run_id="${RUN_ID:-${OPS_RUN_ID:-manual}}"
out="ops/_generated/report.unified.json"
schema="ops/_schemas/report/unified.schema.json"

python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path('.')
run_id = "${run_id}"
out = root / "${out}"
schema_path = root / "${schema}"

lanes = {}
for lane_dir in sorted((root / "ops/_generated").glob("*/")):
    lane = lane_dir.name
    f = lane_dir / f"{run_id}.json"
    if not f.exists():
        continue
    lanes[lane] = json.loads(f.read_text(encoding='utf-8'))

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
required = schema.get("required", [])
for key in required:
    if key not in payload:
        raise SystemExit(f"missing required unified-report field: {key}")

out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding='utf-8')
print(out)
PY
