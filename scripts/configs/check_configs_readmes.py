#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
  errs=[]
  for d in sorted((ROOT / "configs").iterdir()):
    if not d.is_dir() or d.name.startswith("_"):
      continue
    if not (d / "README.md").exists():
      errs.append(f"missing README: configs/{d.name}/README.md")
  if errs:
    print("configs readme coverage failed")
    for e in errs:
      print(f"- {e}")
    return 1
  print("configs readme coverage passed")
  return 0

if __name__ == "__main__":
  raise SystemExit(main())
