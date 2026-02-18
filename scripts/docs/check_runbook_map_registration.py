#!/usr/bin/env python3
# Purpose: enforce that every runbook is registered in the dashboard/alert runbook map.
# Inputs: docs/operations/runbooks/*.md and docs/operations/observability/runbook-dashboard-alert-map.md.
# Outputs: non-zero exit on missing runbook map rows.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUNBOOK_DIR = ROOT / "docs" / "operations" / "runbooks"
RUNBOOK_MAP = ROOT / "docs" / "operations" / "observability" / "runbook-dashboard-alert-map.md"


def main() -> int:
    runbooks = sorted(p.name for p in RUNBOOK_DIR.glob("*.md") if p.name != "INDEX.md")
    map_text = RUNBOOK_MAP.read_text(encoding="utf-8")
    mapped = set(re.findall(r"\|\s*`([^`]+\.md)`\s*\|", map_text))
    missing = [name for name in runbooks if name not in mapped]
    if missing:
        print("runbook map registration check failed:", file=sys.stderr)
        for name in missing:
            print(f"- {name} missing from {RUNBOOK_MAP.relative_to(ROOT)}", file=sys.stderr)
        return 1
    print("runbook map registration check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
