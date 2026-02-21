#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]


def _load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def _exists(rel: str) -> bool:
    return (ROOT / rel).exists()


def main() -> int:
    errors: list[str] = []

    make_ownership = _load(ROOT / "makefiles" / "ownership.json")
    for target, meta in sorted(make_ownership.items()):
        if not meta.get("owner") or not meta.get("area"):
            errors.append(f"make target ownership incomplete: {target}")

    scripts_ownership = _load(ROOT / "scripts" / "areas" / "_meta" / "ownership.json").get("areas", {})
    for rel, owner in sorted(scripts_ownership.items()):
        if not owner:
            errors.append(f"scripts ownership missing owner: {rel}")

    ops_ownership = _load(ROOT / "ops" / "_meta" / "ownership.json").get("areas", {})
    for rel, owner in sorted(ops_ownership.items()):
        if not owner:
            errors.append(f"ops ownership missing owner: {rel}")
        if not _exists(rel):
            errors.append(f"ops ownership path missing: {rel}")

    cfg_ownership = _load(ROOT / "configs" / "meta" / "ownership.json")
    for command, owner in sorted(cfg_ownership.get("commands", {}).items()):
        if not owner:
            errors.append(f"configs ownership command missing owner: {command}")
    for rel, owner in sorted(cfg_ownership.get("paths", {}).items()):
        if not owner:
            errors.append(f"configs ownership path missing owner: {rel}")
        if not _exists(rel):
            errors.append(f"configs ownership path missing: {rel}")

    if errors:
        print("orphan owners check failed:", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("orphan owners check passed (ownership coverage 100%)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
