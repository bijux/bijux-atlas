#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "configs/ops/slo/slo.v1.json"
DST = ROOT / "configs/slo/slo.json"


def main() -> int:
  data = json.loads(SRC.read_text(encoding="utf-8"))
  expected = {
    "schema_version": data.get("schema_version", 1),
    "source": "configs/ops/slo/slo.v1.json",
    "slis": data.get("slis", []),
    "slos": data.get("slos", []),
    "change_policy": data.get("change_policy", {}),
  }
  current = json.loads(DST.read_text(encoding="utf-8"))
  if current != expected:
    print("SLO config drift: configs/slo/slo.json must be generated from configs/ops/slo/slo.v1.json")
    return 1
  print("SLO sync check passed")
  return 0

if __name__=="__main__":
  raise SystemExit(main())
