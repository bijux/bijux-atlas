#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SCORECARD = ROOT / "ops/_generated/scorecard.json"
OUT = ROOT / "docs/_generated/ops-badge.md"


def main() -> int:
    status = "unknown"
    score = 0
    if SCORECARD.exists():
        payload = json.loads(SCORECARD.read_text(encoding="utf-8"))
        status = str(payload.get("status", "unknown"))
        score = int(payload.get("score", 0))

    color = "red"
    if status == "pass":
        color = "brightgreen"
    elif status == "unknown":
        color = "lightgrey"

    lines = [
        "# Ops Badge",
        "",
        f"![ops confidence](https://img.shields.io/badge/ops%20confidence-{status}%20({score}%25)-{color})",
        "",
        f"Source: `{SCORECARD.relative_to(ROOT)}`",
        "",
    ]
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
