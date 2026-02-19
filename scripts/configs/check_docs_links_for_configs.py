#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOCS = (ROOT / "docs").rglob("*.md")
REQUIRED = [
  "configs/security/README.md",
  "configs/rust/README.md",
]


def main() -> int:
  corpus="\n".join(p.read_text(encoding="utf-8", errors="ignore") for p in DOCS)
  missing=[]
  for item in REQUIRED:
    if item not in corpus:
      missing.append(item)
  if missing:
    print("configs docs link check failed")
    for m in missing:
      print(f"- missing docs reference: {m}")
    return 1
  print("configs docs link check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
