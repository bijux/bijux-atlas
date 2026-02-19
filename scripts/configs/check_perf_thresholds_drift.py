#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CFG = ROOT / "configs/perf/k6-thresholds.v1.json"
OPS = ROOT / "ops/load/contracts/k6-thresholds.v1.json"


def main() -> int:
  cfg = json.loads(CFG.read_text(encoding="utf-8"))
  ops = json.loads(OPS.read_text(encoding="utf-8"))
  if cfg != ops:
    print("k6 thresholds drift: configs/perf/k6-thresholds.v1.json != ops/load/contracts/k6-thresholds.v1.json")
    return 1
  print("k6 thresholds drift check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
