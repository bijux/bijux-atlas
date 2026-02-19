#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"
ops_init_run_id

BASE="${ATLAS_BASE_URL:-http://127.0.0.1:8080}"
TOP_N="${TOP_N:-5}"
OUT="$(ops_artifact_dir warm-top)"

resp="$OUT/datasets.json"
ops_retry 3 1 curl -fsS "$BASE/v1/datasets" >"$resp"
datasets="$(python3 - "$resp" "$TOP_N" <<'PY'
import json,sys
p=sys.argv[1]; n=int(sys.argv[2])
obj=json.load(open(p))
rows=[]
if isinstance(obj, dict):
    if isinstance(obj.get("datasets"), list):
        rows=obj["datasets"]
    elif isinstance(obj.get("data"), list):
        rows=obj["data"]
for d in rows[:n]:
    ds=d.get("dataset") if isinstance(d,dict) else None
    if isinstance(ds,dict):
        r,s,a=ds.get("release"),ds.get("species"),ds.get("assembly")
        if r and s and a:
            print(f"{r}/{s}/{a}")
PY
)"
datasets="$(echo "$datasets" | paste -sd, -)"
[ -n "$datasets" ] || { echo "no datasets resolved from /v1/datasets"; exit 1; }
DATASETS="$datasets" "$ROOT/e2e/runner/warm_datasets.sh"
