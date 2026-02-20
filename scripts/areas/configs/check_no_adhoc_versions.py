#!/usr/bin/env python3
from __future__ import annotations
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ALLOW = {
  "configs/ops/tool-versions.json",
  "configs/_schemas/tool-versions.schema.json",
  "ops/stack/versions.json",
  "ops/stack/version-manifest.json",
  "ops/_schemas/stack/version-manifest.schema.json",
}


def main() -> int:
  errs=[]
  version_name = re.compile(r".*version[s]?(?:[-_.].*)?\.(json|yaml|yml|toml)$", re.IGNORECASE)
  for scope in (ROOT / "configs", ROOT / "ops"):
    if not scope.exists():
      continue
    for p in scope.rglob("*"):
      if not p.is_file():
        continue
      rel = p.relative_to(ROOT).as_posix()
      if not version_name.match(p.name):
        continue
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
