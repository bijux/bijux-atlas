#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]
SSOT = ROOT / "configs" / "ops" / "public-make-targets.json"

FORBIDDEN = re.compile(r"\b(phase|step|task|stage)\b", re.I)

def main() -> int:
    data = json.loads(SSOT.read_text(encoding="utf-8"))
    errors: list[str] = []

    for item in data.get("public_targets", []):
        name = item["name"]
        desc = (item.get("description") or "").strip()
        if not desc:
            errors.append(f"{name}: missing description")
            continue
        if len(desc) > 80:
            errors.append(f"{name}: description longer than 80 chars")
        if FORBIDDEN.search(desc):
            errors.append(f"{name}: description uses forbidden wording (phase/step/task/stage)")
        first = re.split(r"\s+", desc)[0]
        if first.lower().endswith("ing"):
            errors.append(f"{name}: description should be imperative, not gerund form ('{first}')")

    if errors:
        print("public target description lint failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("public target description lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
