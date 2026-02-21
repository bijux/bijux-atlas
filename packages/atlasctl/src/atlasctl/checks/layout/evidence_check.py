#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
EVIDENCE = ROOT / "ops" / "_evidence"


def validate_lane_report(path: Path) -> list[str]:
    issues: list[str] = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        return [f"{path.relative_to(ROOT)}: invalid json: {exc}"]
    for key in ("lane", "run_id", "status", "log", "duration_seconds"):
        if key not in data:
            issues.append(f"{path.relative_to(ROOT)}: missing `{key}`")
    if data.get("status") not in {"pass", "fail"}:
        issues.append(f"{path.relative_to(ROOT)}: invalid status `{data.get('status')}`")
    return issues


def validate_unified(path: Path) -> list[str]:
    issues: list[str] = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        return [f"{path.relative_to(ROOT)}: invalid json: {exc}"]
    for key in ("run_id", "generated_at", "lanes", "summary"):
        if key not in data:
            issues.append(f"{path.relative_to(ROOT)}: missing `{key}`")
    return issues


def main() -> int:
    issues: list[str] = []
    if not EVIDENCE.exists():
        print("evidence schema check passed")
        return 0
    for path in EVIDENCE.rglob("report.json"):
        issues.extend(validate_lane_report(path))
    for path in EVIDENCE.rglob("unified.json"):
        issues.extend(validate_unified(path))
    if issues:
        print("evidence schema check failed", file=sys.stderr)
        for issue in issues:
            print(f"- {issue}", file=sys.stderr)
        return 1
    print("evidence schema check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
