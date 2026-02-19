#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SHIMS = ["deny.toml","audit-allowlist.toml","clippy.toml","rustfmt.toml","nextest.toml","Dockerfile"]


def main() -> int:
  errs=[]
  for name in SHIMS:
    p=ROOT / name
    if not p.exists():
      continue
    if not p.is_symlink():
      errs.append(f"{name} must be a symlink shim")
      continue
    target = (p.parent / p.readlink()).resolve()
    if not target.exists():
      errs.append(f"{name} is a broken symlink")
      continue
    if "configs" not in target.parts and name != "Dockerfile":
      errs.append(f"{name} must point into configs/*")
  if errs:
    print("root config shims check failed")
    for e in errs:
      print(f"- {e}")
    return 1
  print("root config shims check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
