#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def main() -> int:
    s = load_surface()
    declared: set[str] = set()
    for mk_path in sorted((ROOT / "makefiles").glob("*.mk")):
        mk = mk_path.read_text(encoding="utf-8")
        declared.update(re.findall(r"^([a-zA-Z0-9_.-]+):(?:\s|$)", mk, flags=re.M))

    missing = [t for t in s["make_targets"] if t not in declared and t != "help"]
    if missing:
        print("public surface check failed: targets not declared in root.mk")
        for m in missing:
            print(f"- {m}")
        return 1

    for cmd in s["ops_run_commands"]:
        if not (ROOT / cmd).exists():
            print(f"public surface check failed: missing command path {cmd}")
            return 1

    print("public surface check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
