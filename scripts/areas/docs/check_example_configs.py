#!/usr/bin/env python3
# Purpose: validate docs example configs against policy schema required keys.
# Inputs: docs/examples/policy-config.example.json and docs/contracts/POLICY_SCHEMA.json
# Outputs: non-zero exit when required fields are missing
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
example = json.loads((ROOT / "docs" / "examples" / "policy-config.example.json").read_text())
schema = json.loads((ROOT / "docs" / "contracts" / "POLICY_SCHEMA.json").read_text())

required = set(schema.get("required", []))
missing = sorted(required - set(example.keys()))
extra = sorted(set(example.keys()) - set(schema.get("properties", {}).keys()))

if missing:
    print(f"example config validation failed: missing keys {missing}")
    raise SystemExit(1)
if extra:
    print(f"example config validation failed: unknown keys {extra}")
    raise SystemExit(1)

print("example config validation passed")
