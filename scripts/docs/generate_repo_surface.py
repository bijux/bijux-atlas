#!/usr/bin/env python3
# Purpose: generate repository navigation surface from SSOT targets and top-level areas.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs" / "_generated" / "repo-surface.md"
SURFACE = ROOT / "configs" / "ops" / "public-surface.json"

EXCLUDE_DIRS = {".git", ".github", ".cargo", "target", ".idea", "node_modules"}


def main() -> int:
    top_dirs = sorted(
        p.name
        for p in ROOT.iterdir()
        if p.is_dir() and p.name not in EXCLUDE_DIRS and not p.name.startswith(".")
    )
    surface = json.loads(SURFACE.read_text(encoding="utf-8"))
    make_targets = surface.get("make_targets", [])
    ops_cmds = surface.get("ops_run_commands", [])

    lines = [
        "# Repository Surface",
        "",
        "## Top-level Areas",
    ]
    lines.extend([f"- `{d}/`" for d in top_dirs])
    lines.extend([
        "",
        "## Public Make Targets",
    ])
    lines.extend([f"- `make {t}`" for t in make_targets])
    lines.extend([
        "",
        "## Public Ops Run Commands",
    ])
    lines.extend([f"- `./{c}`" for c in ops_cmds])

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
