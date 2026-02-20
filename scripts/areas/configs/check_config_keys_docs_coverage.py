#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REGISTRY = ROOT / "configs/config-key-registry.md"
DOC = ROOT / "docs/contracts/config-keys.md"
INTERNAL = ROOT / "configs/ops/internal-config-keys.txt"


def _extract_keys(text: str) -> set[str]:
    return set(re.findall(r"`(ATLAS_[A-Z0-9_]+|BIJUX_[A-Z0-9_]+|HOME|HOSTNAME|XDG_CACHE_HOME|XDG_CONFIG_HOME|REDIS_URL)`", text))


def main() -> int:
    registry_keys = _extract_keys(REGISTRY.read_text(encoding="utf-8"))
    doc_text = DOC.read_text(encoding="utf-8")
    internal_keys = {
        line.strip()
        for line in INTERNAL.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }

    missing = sorted(k for k in registry_keys if k not in internal_keys and f"`{k}`" not in doc_text)
    if missing:
        print("config key docs coverage check failed", file=sys.stderr)
        for key in missing:
            print(f"- missing docs coverage for key: {key}", file=sys.stderr)
        return 1

    print("config key docs coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
