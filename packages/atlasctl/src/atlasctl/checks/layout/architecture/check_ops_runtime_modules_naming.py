#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
RUNTIME_MODS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules"
ALLOWED = {
    "api.py",
    "actions_inventory.py",
    "layer_contract.py",
    "ports.py",
}


def main() -> int:
    errs: list[str] = []
    for path in sorted(RUNTIME_MODS.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(ROOT).as_posix()
        if path.name == "__init__.py":
            continue
        if "assets" in path.parts:
            continue
        if path.name in ALLOWED:
            continue
        if not path.name.startswith("ops_runtime_"):
            errs.append(
                f"{rel}: runtime_modules python files must use `ops_runtime_*` prefix "
                "(or be explicitly allowlisted)"
            )
    if errs:
        print("\n".join(errs))
        return 1
    print("ops runtime_modules naming OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
