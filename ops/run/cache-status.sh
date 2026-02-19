#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"

ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-cache-status"
ops_version_guard kind kubectl

mode="${1:-report}"
./ops/datasets/scripts/sh/cache_status.sh
python3 ./ops/datasets/scripts/py/cache_budget_check.py
if [ "${CACHE_STATUS_STRICT:-1}" = "1" ]; then
  python3 ./ops/datasets/scripts/py/cache_threshold_check.py
fi

if [ "$mode" = "--plan" ]; then
  python3 - <<'PY'
from __future__ import annotations
import json
from pathlib import Path

root = Path.cwd()
manifest = json.loads((root / "ops/datasets/manifest.json").read_text(encoding="utf-8"))
missing: list[str] = []
present: list[str] = []
for ds in manifest.get("datasets", []):
    name = str(ds.get("name", ""))
    dsid = str(ds.get("id", ""))
    parts = dsid.split("/")
    if len(parts) != 3:
        continue
    release, species, assembly = parts
    store_dir = root / "artifacts/e2e-store" / f"release={release}" / f"species={species}" / f"assembly={assembly}"
    if store_dir.exists():
        present.append(name)
    else:
        missing.append(name)
print("cache_plan_present=" + ",".join(present))
print("cache_plan_would_fetch=" + ",".join(missing))
PY
fi
