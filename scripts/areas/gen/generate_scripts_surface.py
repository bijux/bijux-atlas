#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
out = ROOT / "docs/_generated/scripts-surface.md"

lines = [
    "# Scripts Surface",
    "",
    "Generated file. Do not edit manually.",
    "",
    "Scripts are internal unless listed in `configs/ops/public-surface.json` or `scripts/areas/docs/ENTRYPOINTS.md` public section.",
    "",
    "## Script Domains",
    "",
    "- `scripts/areas/check/`: validators and lint gates",
    "- `scripts/areas/gen/`: inventory/document generators",
    "- `scripts/lib/`: reusable shell/python libraries",
    "- `scripts/bin/`: thin entrypoints",
    "",
    "## scripts/bin",
    "",
]

for p in sorted((ROOT / "scripts/bin").glob("*")):
    if p.is_file():
        lines.append(f"- `{p.relative_to(ROOT).as_posix()}`")

lines.extend(["", "## checks", ""])
for p in sorted((ROOT / "scripts/areas/check").glob("*")):
    if p.is_file():
        lines.append(f"- `{p.relative_to(ROOT).as_posix()}`")

lines.extend(["", "## root bin shims", ""])
bin_root = ROOT / "bin"
if bin_root.exists():
    for p in sorted(bin_root.glob("*")):
        if p.is_file():
            lines.append(f"- `{p.relative_to(ROOT).as_posix()}`")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(out)
