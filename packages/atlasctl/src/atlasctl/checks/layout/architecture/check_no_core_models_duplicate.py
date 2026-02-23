#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
DUP = ROOT / "packages/atlasctl/src/atlasctl/core/models"


def main() -> int:
    if DUP.exists():
        print("forbidden duplicate package exists: packages/atlasctl/src/atlasctl/core/models")
        return 1
    offenders: list[str] = []
    for path in (ROOT / "packages/atlasctl").rglob("*.py"):
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "atlasctl.core.models" in text or ".core.models" in text:
            offenders.append(path.relative_to(ROOT).as_posix())
    if offenders:
        print("forbidden imports/usages of atlasctl.core.models found:")
        for rel in offenders:
            print(f"- {rel}")
        return 1
    print("core/models duplicate package retired; no references found")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
