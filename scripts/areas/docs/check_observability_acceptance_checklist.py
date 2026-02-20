#!/usr/bin/env python3
# owner: platform
# purpose: validate observability acceptance checklist structure for release gating.
# stability: public
# called-by: .github/workflows/release.yml
from __future__ import annotations

from pathlib import Path
import sys

root = Path(__file__).resolve().parents[3]
path = root / "docs/operations/observability/acceptance-checklist.md"
if not path.exists():
    print(f"missing checklist: {path}", file=sys.stderr)
    raise SystemExit(1)
text = path.read_text(encoding="utf-8")
required = [
    "## Required Checks",
    "## Release Notes",
    "make telemetry-verify",
    "make observability-pack-drills",
]
missing = [item for item in required if item not in text]
if missing:
    print("acceptance checklist missing required entries:", file=sys.stderr)
    for item in missing:
        print(f"- {item}", file=sys.stderr)
    raise SystemExit(1)
print("observability acceptance checklist contract passed")
