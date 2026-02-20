#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "configs/ops/slo/slo.v1.json"
DST = ROOT / "configs/slo/slo.json"


def rendered_payload() -> dict:
  data = json.loads(SRC.read_text(encoding="utf-8"))
  return {
    "schema_version": data.get("schema_version", 1),
    "source": "configs/ops/slo/slo.v1.json",
    "slis": data.get("slis", []),
    "slos": data.get("slos", []),
    "change_policy": data.get("change_policy", {}),
  }


def main() -> int:
  legacy = rendered_payload()
  DST.write_text(json.dumps(legacy, indent=2) + "\n", encoding="utf-8")
  print(DST)
  return 0

if __name__=="__main__":
  raise SystemExit(main())
