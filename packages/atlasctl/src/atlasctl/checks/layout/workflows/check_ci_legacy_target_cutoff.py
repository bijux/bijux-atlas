#!/usr/bin/env python3
# Purpose: enforce hard cutoff date for legacy make targets in CI workflows.
from __future__ import annotations

import datetime as dt
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
WF = ROOT / ".github" / "workflows"
LEGACY = ROOT / "configs" / "ops" / "nonroot-legacy-targets.txt"
CUTOFF_DATE = dt.date(2026, 3, 1)
RUN_RE = re.compile(r"run:\s*make\s+([^\n]+)")


def _legacy_targets() -> set[str]:
    rows = set()
    for line in LEGACY.read_text(encoding="utf-8").splitlines():
        item = line.strip()
        if not item or item.startswith("#") or ":" not in item:
            continue
        _, target = item.split(":", 1)
        target = target.strip()
        if target:
            rows.add(target)
    return rows


def main() -> int:
    legacy_targets = _legacy_targets()
    offenders: list[str] = []
    for workflow in sorted(WF.glob("*.yml")):
        text = workflow.read_text(encoding="utf-8")
        for run in RUN_RE.findall(text):
            target = run.strip().split()[0]
            if target in legacy_targets:
                offenders.append(f"{workflow.name}: {target}")

    if not offenders:
        print("ci legacy target cutoff check passed")
        return 0

    today = dt.datetime.now(dt.timezone.utc).date()
    if today < CUTOFF_DATE:
        print(
            f"ci legacy target cutoff pending: found {len(offenders)} legacy target references before {CUTOFF_DATE.isoformat()}",
            file=sys.stderr,
        )
        for offender in offenders:
            print(f"- {offender}", file=sys.stderr)
        return 0

    print("ci legacy target cutoff check failed", file=sys.stderr)
    print(f"- cutoff date reached: {CUTOFF_DATE.isoformat()}", file=sys.stderr)
    for offender in offenders:
        print(f"- {offender}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
