#!/usr/bin/env python3
# Purpose: generate ops/stack/versions.json from configs/ops/tool-versions.json SSOT.
# Inputs: configs/ops/tool-versions.json.
# Outputs: ops/stack/versions.json.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
src = ROOT / "configs/ops/tool-versions.json"
out = ROOT / "ops/stack/versions.json"
data = json.loads(src.read_text(encoding="utf-8"))
out.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out)
