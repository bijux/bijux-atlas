#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RENAMES = ROOT / "configs/ops/target-renames.json"
OUT = ROOT / "docs/_generated/upgrade-guide.md"


def main() -> int:
    payload = json.loads(RENAMES.read_text(encoding="utf-8"))
    rows = payload.get("renames", [])
    lines = [
        "# Make Target Upgrade Guide",
        "",
        "Use this table to migrate renamed or aliased make targets.",
        "",
        "| Old Target | New Target | Status |",
        "|---|---|---|",
    ]
    for row in rows:
        lines.append(f"| `{row['from']}` | `{row['to']}` | `{row['status']}` |")
    lines.append("")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(OUT.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
