#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import json
from pathlib import Path

doc = json.loads((Path("ops/k8s/tests/suites.json")).read_text(encoding="utf-8"))
suite = next((s for s in doc["suites"] if s.get("id") == "resilience"), None)
if suite is None:
    raise SystemExit("resilience suite missing")
budget = int(suite.get("budget_minutes", 0))
if budget <= 0 or budget > 30:
    raise SystemExit("resilience suite budget must be set and <= 30 minutes")
print("resilience suite budget contract passed")
PY
