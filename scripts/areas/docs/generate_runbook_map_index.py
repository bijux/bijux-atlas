#!/usr/bin/env python3
# Purpose: generate docs/_generated/runbook-map-index.md from runbooks and runbook map.
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
RUNBOOK_DIR = ROOT / "docs" / "operations" / "runbooks"
MAP_DOC = ROOT / "docs" / "operations" / "observability" / "runbook-dashboard-alert-map.md"
OUT = ROOT / "docs" / "_generated" / "runbook-map-index.md"


def main() -> int:
    runbooks = sorted(
        p.name for p in RUNBOOK_DIR.glob("*.md") if p.name != "INDEX.md"
    )
    map_text = MAP_DOC.read_text(encoding="utf-8")
    mapped = set(re.findall(r"\|\s*`([^`]+\.md)`\s*\|", map_text))
    lines = [
        "# Runbook Map Index (Generated)",
        "",
        "Generated from runbooks and observability runbook map.",
        "",
        f"- Total runbooks: `{len(runbooks)}`",
        f"- Mapped runbooks: `{len([r for r in runbooks if r in mapped])}`",
        "",
        "| Runbook | In map |",
        "|---|---|",
    ]
    for name in runbooks:
        lines.append(f"| `{name}` | {'yes' if name in mapped else 'no'} |")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
