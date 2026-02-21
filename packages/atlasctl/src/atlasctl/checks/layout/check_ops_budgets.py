#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
BUDGETS = ROOT / "configs/ops/budgets.json"
SCHEMA = ROOT / "ops/_schemas/meta/budgets.schema.json"
DOC = ROOT / "docs/operations/performance/budgets.md"


def _require(cond: bool, msg: str, errors: list[str]) -> None:
    if not cond:
        errors.append(msg)


def main() -> int:
    errors: list[str] = []
    payload = json.loads(BUDGETS.read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))

    for key in schema.get("required", []):
        _require(key in payload, f"missing required budget key `{key}`", errors)

    if DOC.exists():
        text = DOC.read_text(encoding="utf-8")
        for key in ("smoke", "root_local", "k6_latency", "cold_start", "cache", "metric_cardinality"):
            _require(key in text, f"budget docs drift: `{key}` not documented in {DOC.relative_to(ROOT)}", errors)
    else:
        errors.append(f"missing budget docs: {DOC.relative_to(ROOT)}")

    lane_budgets = payload.get("root_local", {}).get("lane_max_duration_seconds", {})
    expected_lanes = {
        "lane-cargo",
        "lane-docs",
        "lane-ops",
        "lane-scripts",
        "lane-configs-policies",
        "internal/lane-obs-cheap",
    }
    for lane in sorted(expected_lanes):
        _require(lane in lane_budgets, f"missing root_local budget for lane `{lane}`", errors)

    if errors:
        print("ops budgets check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("ops budgets check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
