#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
cd "$ROOT"

python3 - <<'PY'
import json
from pathlib import Path

caps = json.loads(Path("configs/ops/hpa-safety-caps.json").read_text(encoding="utf-8"))
default_cap = int(caps.get("default_max_replicas_cap", 0))
profile_caps = caps.get("profile_caps", {})
for values in sorted(Path("ops/k8s/values").glob("*.yaml")):
    lines = values.read_text(encoding="utf-8").splitlines()
    max_replicas = None
    in_hpa = False
    for line in lines:
        if line.startswith("hpa:"):
            in_hpa = True
            continue
        if in_hpa and line and not line.startswith(" "):
            in_hpa = False
        if in_hpa and "maxReplicas:" in line:
            try:
                max_replicas = int(line.split(":", 1)[1].strip())
            except ValueError:
                pass
            break
    if max_replicas is None:
        continue
    profile = values.stem
    cap = int(profile_caps.get(profile, default_cap))
    if max_replicas > cap:
        raise SystemExit(f"failure_mode: hpa_max_replicas_cap_exceeded profile={profile} maxReplicas={max_replicas} cap={cap}")
print("hpa max replicas cap contract passed")
PY
