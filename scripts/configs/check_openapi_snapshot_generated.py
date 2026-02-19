#!/usr/bin/env python3
from __future__ import annotations
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
  subprocess.run(["python3","scripts/contracts/gen_openapi.py"], cwd=ROOT, check=True)
  generated = ROOT / "configs/openapi/v1/openapi.generated.json"
  snapshot = ROOT / "configs/openapi/v1/openapi.snapshot.json"
  if generated.read_text(encoding="utf-8") != snapshot.read_text(encoding="utf-8"):
    print("openapi snapshot drift: configs/openapi/v1/openapi.snapshot.json must match generator output")
    return 1
  print("openapi snapshot generation check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
