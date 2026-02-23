#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
ENTRY = ROOT / "packages/atlasctl/src/atlasctl/__main__.py"


def main() -> int:
    text = ENTRY.read_text(encoding="utf-8")
    required = [
        "from .app.main import main",
        "if __name__ == \"__main__\":",
        "raise SystemExit(main())",
    ]
    missing = [line for line in required if line not in text]
    if missing:
        print("main entrypoint contract failed")
        for item in missing:
            print(f"missing: {item}")
        return 1
    print("main entrypoint contract passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
