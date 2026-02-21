#!/usr/bin/env python3
# Purpose: enforce configs/ops/observability-pack.json as observability pack SSOT.
# Inputs: ops/obs scripts/config refs.
# Outputs: non-zero when install path bypasses SSOT config.
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
config = ROOT / "configs/ops/observability-pack.json"
_ = json.loads(config.read_text(encoding="utf-8"))
install = (ROOT / "ops/obs/scripts/install_pack.sh").read_text(encoding="utf-8")
if "configs/ops/observability-pack.json" not in install:
    print("ops/obs/scripts/install_pack.sh must read configs/ops/observability-pack.json", file=sys.stderr)
    raise SystemExit(1)
print("observability-pack SSOT check passed")
