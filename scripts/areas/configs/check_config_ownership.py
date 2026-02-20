#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def main() -> int:
  own = json.loads((ROOT / "configs/_meta/ownership.json").read_text(encoding="utf-8"))
  mapped=set(own.get("areas",{}).keys())
  errs=[]
  for d in sorted((ROOT / "configs").iterdir()):
    if not d.is_dir() or d.name.startswith("_"):
      continue
    rel=f"configs/{d.name}"
    if rel not in mapped:
      errs.append(f"missing ownership mapping: {rel}")
  for m in mapped:
    if not (ROOT / m).exists():
      errs.append(f"ownership points to missing area: {m}")
  if errs:
    print("config ownership coverage failed")
    for e in errs:
      print(f"- {e}")
    return 1
  print("config ownership coverage passed")
  return 0

if __name__ == "__main__":
  raise SystemExit(main())
