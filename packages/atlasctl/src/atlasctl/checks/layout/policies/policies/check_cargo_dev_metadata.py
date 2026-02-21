#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
DEV_MK = ROOT / "makefiles" / "dev.mk"

TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def main() -> int:
    text = DEV_MK.read_text(encoding="utf-8")
    targets = [t for t in TARGET_RE.findall(text) if not t.startswith(".")]
    legacy = {
        "dev-fmt",
        "dev-lint",
        "dev-test",
        "dev-coverage",
        "internal/dev/fmt",
        "internal/dev/lint",
        "internal/dev/test",
        "internal/dev/audit",
        "internal/dev/coverage",
        "internal/dev/ci",
        "ci-core",
    }
    errors: list[str] = [f"legacy dev-* target still present in dev.mk: {t}" for t in targets if t in legacy]

    if errors:
        print("dev wrapper metadata check failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("dev wrapper metadata check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
