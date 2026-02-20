#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "configs/ops/public-surface.json"
ROOT_MK = ROOT / "makefiles/root.mk"


def load_surface() -> dict:
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def main() -> int:
    s = load_surface()
    root_text = ROOT_MK.read_text(encoding="utf-8")
    declared: set[str] = set()
    for mk_path in sorted((ROOT / "makefiles").glob("*.mk")):
        mk = mk_path.read_text(encoding="utf-8")
        declared.update(re.findall(r"^([a-zA-Z0-9_./-]+):(?:\s|$)", mk, flags=re.M))

    phony_targets: set[str] = set()
    for line in root_text.splitlines():
        if line.startswith(".PHONY:"):
            phony_targets.update(line.replace(".PHONY:", "", 1).split())

    missing = [t for t in s["make_targets"] if t not in declared and t != "help"]
    if missing:
        print("public surface check failed: targets are not declared in makefiles/*.mk")
        for m in missing:
            print(f"- {m}")
        return 1

    not_registered: list[str] = []
    for target in s["make_targets"]:
        if target == "help":
            continue
        if target not in phony_targets:
            not_registered.append(target)
    if not_registered:
        print("public surface check failed: public targets must be registered in makefiles/root.mk .PHONY")
        for item in not_registered:
            print(f"- {item}")
        return 1

    for cmd in s["ops_run_commands"]:
        if not (ROOT / cmd).exists():
            print(f"public surface check failed: missing command path {cmd}")
            return 1
    for core in s.get("core_targets", []):
        if core not in s["make_targets"]:
            print(f"public surface check failed: core target must be public: {core}")
            return 1

    print("public surface check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
