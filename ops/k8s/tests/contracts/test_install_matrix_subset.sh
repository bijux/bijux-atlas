#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import json
from pathlib import Path

matrix = json.loads((Path("ops/k8s/install-matrix.json")).read_text(encoding="utf-8"))
profiles = matrix.get("profiles", [])
subset = [p for p in profiles if p.get("suite") == "install-gate"]
if len(subset) == 0:
    raise SystemExit("install matrix subset missing: expected at least one install-gate profile")
if len(subset) > 2:
    raise SystemExit("install matrix subset too large for local runs (max 2 install-gate profiles)")
names = {p.get("name") for p in subset}
if "local" not in names:
    raise SystemExit("install matrix subset must include local profile")
print("install matrix subset contract passed")
PY
