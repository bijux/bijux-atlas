#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    doc = json.loads((root / "ops/k8s/tests/suites.json").read_text(encoding="utf-8"))
    suite = next((s for s in doc["suites"] if s.get("id") == "full"), None)
    if suite is None:
        raise SystemExit("full suite missing")
    budget = int(suite.get("budget_minutes", 0))
    if budget <= 0:
        raise SystemExit("full suite budget must be set")
    if not bool(suite.get("require_progress_logs", False)):
        raise SystemExit("full suite must require progress logs")
    print("full suite budget/progress contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
