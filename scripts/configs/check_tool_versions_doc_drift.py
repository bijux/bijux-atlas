#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "configs/ops/tool-versions.json"
DST = ROOT / "docs/_generated/tooling-versions.md"


def main() -> int:
  data = json.loads(SRC.read_text(encoding="utf-8"))
  lines=["# Tooling Versions","","Generated from `configs/ops/tool-versions.json`.",""]
  for k,v in sorted(data.items()):
    lines.append(f"- `{k}`: `{v}`")
  lines.append("")
  expected = "\n".join(lines)
  current = DST.read_text(encoding="utf-8")
  if current != expected:
    print("tooling versions doc drift detected")
    return 1
  print("tooling versions doc drift check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
