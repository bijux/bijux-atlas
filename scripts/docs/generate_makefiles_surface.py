#!/usr/bin/env python3
# Purpose: generate docs/development/makefiles/surface.md from public surface SSOT.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"
OUT = ROOT / "docs/development/makefiles/surface.md"


def main() -> int:
    data = json.loads(SURFACE.read_text(encoding="utf-8"))
    lines = [
        "# Makefiles Public Surface",
        "",
        "Generated from `configs/ops/public-surface.json`. Do not edit manually.",
        "",
        "## Core Gates",
    ]
    for t in data.get("core_targets", []):
        lines.append(f"- `make {t}`")
    lines.extend(["", "## Public Targets"])
    for t in data.get("make_targets", []):
        lines.append(f"- `make {t}`")
    lines.extend(["", "## Public Ops Run Commands"])
    for c in data.get("ops_run_commands", []):
        lines.append(f"- `./{c}`")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(OUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
