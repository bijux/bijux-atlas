#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUDGET = ROOT / "configs/ops/budgets.json"
RELAX = ROOT / "configs/policy/budget-relaxations.json"


def _allow_exemption(duration: float, maximum: float) -> bool:
    if duration <= maximum:
        return True
    exc_id = os.environ.get("BUDGET_RELAXATION_ID", "").strip()
    if not exc_id or exc_id == "none":
        return False
    payload = json.loads(RELAX.read_text(encoding="utf-8"))
    today = dt.date.today()
    for e in payload.get("exceptions", []):
        if str(e.get("id", "")).strip() != exc_id:
            continue
        if str(e.get("budget_id", "")).strip() != "smoke_duration":
            continue
        expiry = str(e.get("expiry", "")).strip()
        try:
            expiry_date = dt.date.fromisoformat(expiry)
        except ValueError:
            return False
        return expiry_date >= today
    return False


def main() -> int:
    run_id = os.environ.get("RUN_ID", "").strip()
    if not run_id:
        print("RUN_ID is required for ops smoke budget check", file=sys.stderr)
        return 2
    report = ROOT / "artifacts/evidence/ops-smoke" / run_id / "report.json"
    if not report.exists():
        print(f"missing report: {report}", file=sys.stderr)
        return 1
    budget = json.loads(BUDGET.read_text(encoding="utf-8"))
    max_seconds = float(budget.get("smoke", {}).get("max_duration_seconds", 600))
    payload = json.loads(report.read_text(encoding="utf-8"))
    duration = float(payload.get("duration_seconds", 0.0))
    if _allow_exemption(duration, max_seconds):
        print(f"ops smoke budget passed: {duration:.2f}s <= {max_seconds:.2f}s (or valid exemption)")
        return 0
    print(
        f"ops smoke budget failed: duration={duration:.2f}s exceeds {max_seconds:.2f}s; "
        "set BUDGET_RELAXATION_ID with a valid non-expired `smoke_duration` relaxation to bypass",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
