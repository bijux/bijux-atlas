#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MAKEFILES = sorted((ROOT / "makefiles").glob("*.mk"))


def main() -> int:
    errors: list[str] = []
    for path in MAKEFILES:
        lines = path.read_text(encoding="utf-8").splitlines()[:8]
        joined = "\n".join(lines)
        if "# Scope:" not in joined:
            errors.append(f"{path.relative_to(ROOT)}: missing '# Scope:' header")
        if "# Public targets:" not in joined:
            errors.append(f"{path.relative_to(ROOT)}: missing '# Public targets:' header")
        if path.name != "root.mk" and "# Public targets: none" not in joined:
            errors.append(f"{path.relative_to(ROOT)}: non-root .mk must declare '# Public targets: none'")

    if errors:
        print("makefile headers lint failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("makefile headers lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
