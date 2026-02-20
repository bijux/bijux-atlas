#!/usr/bin/env bash
set -euo pipefail
python3 - <<'PY'
import json
from pathlib import Path

doc = json.loads((Path("ops/k8s/tests/suites.json")).read_text(encoding="utf-8"))
suite = next((s for s in doc["suites"] if s.get("id") == "full"), None)
if suite is None:
    raise SystemExit("full suite missing")
budget = int(suite.get("budget_minutes", 0))
if budget <= 0:
    raise SystemExit("full suite budget must be set")
if not bool(suite.get("require_progress_logs", False)):
    raise SystemExit("full suite must require progress logs")
print("full suite budget/progress contract passed")
PY
