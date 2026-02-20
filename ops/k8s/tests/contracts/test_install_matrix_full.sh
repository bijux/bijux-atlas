#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import json
from pathlib import Path

matrix = json.loads((Path("ops/k8s/install-matrix.json")).read_text(encoding="utf-8"))
profiles = matrix.get("profiles", [])
nightly = [p for p in profiles if p.get("suite") == "nightly"]
if len(nightly) < 3:
    raise SystemExit("nightly install matrix must include at least three profiles")
required = {"perf", "multi-registry", "ingress"}
names = {p.get("name") for p in nightly}
missing = sorted(required - names)
if missing:
    raise SystemExit(f"nightly install matrix missing required profiles: {missing}")
print("install matrix full contract passed")
PY
