#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
python3 - <<'PY'
import json
from pathlib import Path

doc = json.loads((Path("ops/k8s/tests/suites.json")).read_text(encoding="utf-8"))
smoke = next((s for s in doc["suites"] if s.get("id") == "smoke"), None)
if smoke is None:
    raise SystemExit("smoke suite missing")
budget = int(smoke.get("budget_minutes", 0))
if budget <= 0 or budget > 15:
    raise SystemExit("smoke suite budget must be set and <= 15 minutes")
print("smoke suite budget contract passed")
PY
