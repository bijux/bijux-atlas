#!/usr/bin/env python3
from __future__ import annotations
import json
from pathlib import Path
import jsonschema

ROOT = Path(__file__).resolve().parents[3]

PAIRS = [
  ("configs/_meta/ownership.json", "configs/_schemas/configs-ownership.schema.json"),
  ("configs/ops/tool-versions.json", "configs/_schemas/tool-versions.schema.json"),
  ("configs/ops/public-surface.json", "configs/_schemas/public-surface.schema.json"),
  ("configs/policy/policy-relaxations.json", "configs/_schemas/policy-relaxations.schema.json"),
  ("configs/policy/layer-relaxations.json", "configs/_schemas/layer-relaxations.schema.json"),
  ("configs/policy/layer-live-diff-allowlist.json", "configs/_schemas/layer-live-diff-allowlist.schema.json"),
  ("configs/ops/target-renames.json", "configs/_schemas/target-renames.schema.json"),
  ("configs/ops/hpa-safety-caps.json", "configs/_schemas/hpa-safety-caps.schema.json"),
  ("configs/meta/ownership.json", "configs/_schemas/meta-ownership.schema.json"),
]


def main() -> int:
  errs=[]
  for data_p, schema_p in PAIRS:
    data = json.loads((ROOT / data_p).read_text(encoding="utf-8"))
    schema = json.loads((ROOT / schema_p).read_text(encoding="utf-8"))
    try:
      jsonschema.validate(data, schema)
    except jsonschema.ValidationError as e:
      errs.append(f"{data_p} vs {schema_p}: {e.message}")
  if errs:
    print("config schema validation failed")
    for e in errs:
      print(f"- {e}")
    return 1
  print("config schema validation passed")
  return 0

if __name__ == "__main__":
  raise SystemExit(main())
