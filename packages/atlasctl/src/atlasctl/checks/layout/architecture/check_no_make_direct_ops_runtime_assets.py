#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
FORBID = "packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/"


def main() -> int:
    errs: list[str] = []
    for path in sorted((ROOT / "makefiles").rglob("*.mk")):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if FORBID in text and "$(ATLASCTL)" not in text:
            errs.append(f"{rel}: direct runtime asset script usage forbidden")
    if errs:
        print("\n".join(errs))
        return 1
    print("makefiles do not call ops runtime asset scripts directly")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
