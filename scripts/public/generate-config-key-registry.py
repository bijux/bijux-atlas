#!/usr/bin/env python3
# owner: platform
# purpose: generate config key registry markdown from contract SSOT.
# stability: public
# called-by: make config-validate
# Inputs: docs/contracts/CONFIG_KEYS.json
# Outputs: configs/config-key-registry.md
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "docs" / "contracts" / "CONFIG_KEYS.json"
OUT = ROOT / "configs" / "config-key-registry.md"


def main() -> int:
    payload = json.loads(SRC.read_text(encoding="utf-8"))
    keys = sorted(payload.get("env_keys", []))
    lines = [
        "# Config Key Registry",
        "",
        "Generated file. Do not edit manually.",
        "",
        "- Source: `docs/contracts/CONFIG_KEYS.json`",
        f"- Count: `{len(keys)}`",
        "",
        "## Keys",
        "",
    ]
    lines.extend(f"- `{k}`" for k in keys)
    lines.append("")
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"generated {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
