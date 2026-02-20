#!/usr/bin/env python3
from __future__ import annotations

import datetime as dt
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
LEGACY_MK = ROOT / "makefiles/legacy.mk"
DATE_RE = re.compile(r"^LEGACY_REMOVAL_DATE\s*:?=\s*([0-9]{4}-[0-9]{2}-[0-9]{2})$", re.M)
TARGET_RE = re.compile(r"^(legacy/[A-Za-z0-9_./-]+):\s*##\s*(.+)$", re.M)


def main() -> int:
    text = LEGACY_MK.read_text(encoding="utf-8")
    errs: list[str] = []

    m = DATE_RE.search(text)
    if not m:
        errs.append("makefiles/legacy.mk must define LEGACY_REMOVAL_DATE:=YYYY-MM-DD")
    else:
        removal = dt.date.fromisoformat(m.group(1))
        today = dt.date.today()
        if today > removal:
            errs.append(f"legacy targets expired on {removal.isoformat()} and must be removed")

    if "DEPRECATION:" not in text:
        errs.append("makefiles/legacy.mk must include a DEPRECATION banner")

    for target, desc in TARGET_RE.findall(text):
        if "DEPRECATED" not in desc.upper():
            errs.append(f"{target} description must include DEPRECATED marker")

    if errs:
        print("legacy deprecation contract failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("legacy deprecation contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
