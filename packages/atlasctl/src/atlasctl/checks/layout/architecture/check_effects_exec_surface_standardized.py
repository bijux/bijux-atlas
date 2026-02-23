#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
SRC = ROOT / "packages/atlasctl/src/atlasctl"


def main() -> int:
    errs: list[str] = []
    legacy = SRC / "core/effects/exec.py"
    if legacy.exists():
        errs.append("forbidden legacy shim exists: packages/atlasctl/src/atlasctl/core/effects/exec.py")
    self_path = Path(__file__).resolve()
    for path in sorted(SRC.rglob("*.py")):
        if path.resolve() == self_path:
            continue
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "atlasctl.core.effects.exec" in text or "from ..core.effects import exec" in text:
            errs.append(f"{rel}: forbidden import of legacy core.effects.exec shim")
    if errs:
        print("\n".join(errs))
        return 1
    print("effects exec surface standardized")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
