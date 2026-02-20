#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ALLOW = {
  "configs/ops/tool-versions.json",
  "configs/_schemas/tool-versions.schema.json",
  "ops/stack/versions.json",
  "ops/stack/version-manifest.json",
}


def main() -> int:
  errs=[]
  for p in ROOT.rglob("*versions*.json"):
    rel=p.relative_to(ROOT).as_posix()
    if rel in ALLOW or rel.startswith("artifacts/") or rel.startswith("ops/_generated/") or rel.startswith("ops/_generated_committed/") or rel.startswith("artifacts/evidence/"):
      continue
    errs.append(rel)
  if errs:
    print("ad-hoc versions file check failed")
    for e in sorted(errs):
      print(f"- {e}")
    return 1
  print("ad-hoc versions file check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
