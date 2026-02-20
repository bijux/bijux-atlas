#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GEN = ROOT / "ops" / "_meta" / "generate_layer_contract.py"
CONTRACT = ROOT / "ops" / "_meta" / "layer-contract.json"


def main() -> int:
    before = CONTRACT.read_text(encoding="utf-8") if CONTRACT.exists() else ""
    subprocess.run([sys.executable, str(GEN)], cwd=ROOT, check=True)
    after = CONTRACT.read_text(encoding="utf-8")
    if before != after:
        print("layer-contract drift detected: run ops/_meta/generate_layer_contract.py and commit the result", file=sys.stderr)
        return 1
    # normalize check
    json.loads(after)
    print("layer contract drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
