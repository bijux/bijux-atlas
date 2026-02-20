#!/usr/bin/env python3
# Purpose: generate canonical env contract from config key registry.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REGISTRY = ROOT / "configs" / "config-key-registry.md"
OUT = ROOT / "configs" / "contracts" / "env.schema.json"


def parse_keys() -> list[str]:
    keys: list[str] = []
    for line in REGISTRY.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line.startswith("- `") or not line.endswith("`"):
            continue
        key = line[3:-1]
        if key.startswith("ATLAS_") or key.startswith("BIJUX_"):
            keys.append(key)
    keys.append("ATLAS_DEV_ALLOW_UNKNOWN_ENV")
    return sorted(set(keys))


def main() -> int:
    payload = {
        "schema_version": 1,
        "description": "Runtime env allowlist contract for atlas-server; unknown ATLAS_/BIJUX_ keys are rejected unless dev escape hatch is enabled.",
        "enforced_prefixes": ["ATLAS_", "BIJUX_"],
        "dev_mode_allow_unknown_env": "ATLAS_DEV_ALLOW_UNKNOWN_ENV",
        "allowed_env": parse_keys(),
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
