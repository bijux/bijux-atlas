#!/usr/bin/env python3
# Purpose: forbid direct tool tokens in wrapper makefiles (cargo/ci/dev).
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
WRAPPERS = [
    ROOT / "makefiles" / "dev.mk",
    ROOT / "makefiles" / "ci.mk",
]
FORBIDDEN = (
    "cargo ",
    "python3 ",
    "kubectl ",
    "helm ",
    "docker ",
    "k6 ",
    "rm ",
    "make ",
    "pip ",
)


def main() -> int:
    errors: list[str] = []
    for path in WRAPPERS:
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if not line.startswith("\t"):
                continue
            stripped = line.strip().lower()
            for token in FORBIDDEN:
                if re.search(rf"(^|[;&| ]){re.escape(token.strip())}([ ]|$)", stripped):
                    errors.append(f"{path.relative_to(ROOT)}:{lineno}: forbidden token in wrapper recipe: `{token.strip()}`")

    if errors:
        print("make wrapper forbidden-token check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("make wrapper forbidden-token check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
