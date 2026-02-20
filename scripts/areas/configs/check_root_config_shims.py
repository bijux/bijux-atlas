#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FORBIDDEN_ROOT_CONFIGS = ["deny.toml", "audit-allowlist.toml", "clippy.toml", "rustfmt.toml", ".vale", ".vale.ini"]


def main() -> int:
  errs = []
  for name in FORBIDDEN_ROOT_CONFIGS:
    p = ROOT / name
    if p.exists() or p.is_symlink():
      errs.append(f"{name} must not exist at repository root; use explicit config paths under configs/")
  if errs:
    print("root config shims check failed")
    for e in errs:
      print(f"- {e}")
    return 1
  print("root config shims check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
