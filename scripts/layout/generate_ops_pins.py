#!/usr/bin/env python3
# Purpose: generate unified pins SSOT from split configs/ops/pins/*.json files.
# Inputs: configs/ops/pins/tools.json, images.json, helm.json, datasets.json.
# Outputs: configs/ops/pins.json.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PINS_DIR = ROOT / "configs" / "ops" / "pins"
OUT = ROOT / "configs" / "ops" / "pins.json"


def _read(name: str) -> dict:
    return json.loads((PINS_DIR / name).read_text(encoding="utf-8"))


def main() -> int:
    tools = _read("tools.json")
    images = _read("images.json")
    helm = _read("helm.json")
    datasets = _read("datasets.json")
    unified = {
        "schema_version": 1,
        "contract_version": "1.0.0",
        "tools": tools.get("tools", {}),
        "images": images.get("images", {}),
        "helm": helm.get("helm", {}),
        "datasets": datasets.get("datasets", {}),
        "policy": {
            "allow_pin_bypass": False,
            "relaxation_registry": "configs/policy/pin-relaxations.json",
        },
    }
    OUT.write_text(json.dumps(unified, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(OUT.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
