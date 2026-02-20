#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OWN = ROOT / "configs/_meta/ownership.json"
OUT = ROOT / "configs/INDEX.md"


def main() -> int:
    ownership = json.loads(OWN.read_text(encoding="utf-8"))["areas"]
    lines = [
        "# Configs Index",
        "",
        "Canonical configuration surface for repository behavior.",
        "",
        "## Areas",
    ]
    for area in sorted(ownership):
        lines.append(f"- `{area}` owner: `{ownership[area]}`")
    lines.extend(["", "See also: `configs/_meta/ownership.json`.", ""])
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"generated {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
