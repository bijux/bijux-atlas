#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[2]
OWN = json.loads((ROOT / "configs/_meta/ownership.json").read_text(encoding="utf-8"))
OUT = ROOT / "docs/_generated/configs-surface.md"


def main() -> int:
  lines=["# Configs Surface","","Generated from `configs/` structure and ownership map.",""]
  for d in sorted((ROOT / "configs").iterdir()):
    if not d.is_dir() or d.name.startswith("_"):
      continue
    rel=f"configs/{d.name}"
    owner = OWN.get("areas",{}).get(rel,"<unowned>")
    readme = d / "README.md"
    lines.append(f"## `{rel}`")
    lines.append(f"- Owner: `{owner}`")
    lines.append(f"- README: `{readme.relative_to(ROOT).as_posix() if readme.exists() else 'MISSING'}`")
    lines.append("")
  OUT.parent.mkdir(parents=True, exist_ok=True)
  OUT.write_text("\n".join(lines), encoding="utf-8")
  print(OUT)
  return 0

if __name__=="__main__":
  raise SystemExit(main())
