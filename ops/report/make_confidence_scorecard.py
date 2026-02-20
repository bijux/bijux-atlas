#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
BUDGETS = ROOT / "configs/ops/budgets.json"


def _budget(name: str, passed: bool, detail: str) -> dict[str, object]:
    return {"name": name, "status": "pass" if passed else "fail", "detail": detail}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--unified", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--print-summary", action="store_true")
    args = parser.parse_args()

    unified = Path(args.unified)
    payload = json.loads(unified.read_text(encoding="utf-8")) if unified.exists() else {"lanes": {}, "summary": {"total": 0, "passed": 0, "failed": 0}}
    lanes = payload.get("lanes", {})
    summary = payload.get("summary", {"total": 0, "passed": 0, "failed": 0})

    budget_cfg = json.loads(BUDGETS.read_text(encoding="utf-8")) if BUDGETS.exists() else {}
    smoke_limit = int(budget_cfg.get("smoke", {}).get("max_duration_seconds", 600))
    obs_lane = lanes.get("internal/lane-obs-cheap") or lanes.get("lane-obs-cheap")
    ops_smoke = lanes.get("internal/lane-ops-smoke") or lanes.get("lane-ops-smoke")
    smoke_duration = int(float(ops_smoke.get("duration_seconds", 0))) if isinstance(ops_smoke, dict) else 0

    budgets = [
        _budget("all-lanes-green", int(summary.get("failed", 0)) == 0 and int(summary.get("total", 0)) > 0, f"failed={summary.get('failed', 0)} total={summary.get('total', 0)}"),
        _budget("obs-cheap-green", isinstance(obs_lane, dict) and obs_lane.get("status") == "pass", "requires internal/lane-obs-cheap passing"),
        _budget("ops-smoke-time-budget", isinstance(ops_smoke, dict) and ops_smoke.get("status") == "pass" and smoke_duration <= smoke_limit, f"duration_sec={smoke_duration}, limit={smoke_limit}"),
    ]
    passed = sum(1 for b in budgets if b["status"] == "pass")

    scorecard = {
        "schema_version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "run_id": payload.get("run_id", "unknown"),
        "summary": summary,
        "budgets": budgets,
        "score": int((passed / len(budgets)) * 100),
        "status": "pass" if passed == len(budgets) else "fail",
    }

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(scorecard, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if args.print_summary:
        print(f"confidence scorecard: status={scorecard['status']} score={scorecard['score']}")
        for b in budgets:
            print(f"- {b['name']}: {b['status']} ({b['detail']})")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
