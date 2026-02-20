#!/usr/bin/env python3
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUDGETS = ROOT / "configs/ops/budgets.json"
RELAX = ROOT / "configs/policy/budget-relaxations.json"


def _find_relaxation(relax_id: str, budget_id: str) -> bool:
    if not relax_id:
        return False
    payload = json.loads(RELAX.read_text(encoding="utf-8"))
    today = dt.date.today()
    for e in payload.get("exceptions", []):
        if str(e.get("id", "")).strip() != relax_id:
            continue
        if str(e.get("budget_id", "")).strip() != budget_id:
            continue
        try:
            expiry = dt.date.fromisoformat(str(e.get("expiry", "")).strip())
        except ValueError:
            return False
        return expiry >= today
    return False


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--lane", required=True)
    p.add_argument("--duration-seconds", type=float, required=True)
    args = p.parse_args()

    payload = json.loads(BUDGETS.read_text(encoding="utf-8"))
    limits = payload.get("root_local", {}).get("lane_max_duration_seconds", {})
    warning_ratio = float(payload.get("root_local", {}).get("warning_band_ratio", 0.9))
    lane = args.lane
    duration = float(args.duration_seconds)
    max_seconds = limits.get(lane)

    result = {
        "lane": lane,
        "checked": max_seconds is not None,
        "duration_seconds": duration,
        "max_seconds": max_seconds,
        "warning_ratio": warning_ratio,
        "near_failing": False,
        "status": "pass",
        "detail": "no budget configured",
    }

    if max_seconds is None:
        print(json.dumps(result, sort_keys=True))
        return 0

    budget_id = f"lane_duration:{lane}"
    relax_id = os.environ.get("BUDGET_RELAXATION_ID", "").strip()
    near_threshold = float(max_seconds) * warning_ratio
    result["near_failing"] = duration >= near_threshold

    if duration <= float(max_seconds):
        result["detail"] = f"{duration:.2f}s <= {float(max_seconds):.2f}s"
        print(json.dumps(result, sort_keys=True))
        return 0

    if _find_relaxation(relax_id, budget_id):
        result["status"] = "pass"
        result["detail"] = (
            f"{duration:.2f}s > {float(max_seconds):.2f}s but allowed by relaxation `{relax_id}`"
        )
        print(json.dumps(result, sort_keys=True))
        return 0

    result["status"] = "fail"
    result["detail"] = (
        f"{duration:.2f}s exceeds {float(max_seconds):.2f}s "
        f"(set BUDGET_RELAXATION_ID for active {budget_id} relaxation)"
    )
    print(json.dumps(result, sort_keys=True))
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
