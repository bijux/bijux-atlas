#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

SCAN = [ROOT / "makefiles", ROOT / "docs", ROOT / "ops" / "run", ROOT / "ops" / "e2e"]
ALLOWED = {
    ROOT / "ops" / "e2e" / "runner" / "suite.sh",
    ROOT / "ops" / "run" / "e2e.sh",
    ROOT / "ops" / "e2e" / "suites" / "suites.json",
    ROOT / "ops" / "e2e" / "realdata" / "scenarios.json",
}
PATTERN = re.compile(r"ops/e2e/realdata/[a-z_]+\.sh")


def main() -> int:
    errors: list[str] = []
    for base in SCAN:
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            if path.suffix not in {".sh", ".mk", ".md", ".json", ".py"}:
                continue
            if path in ALLOWED:
                continue
            if path.match("*/ops/e2e/realdata/*.sh"):
                # scenario implementation scripts may reference each other internally.
                continue
            text = path.read_text(encoding="utf-8", errors="ignore")
            for m in PATTERN.finditer(text):
                errors.append(f"{path.relative_to(ROOT)}: direct realdata scenario usage forbidden: {m.group(0)}")

    if errors:
        print("direct e2e scenario usage lint failed", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print("direct e2e scenario usage lint passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
