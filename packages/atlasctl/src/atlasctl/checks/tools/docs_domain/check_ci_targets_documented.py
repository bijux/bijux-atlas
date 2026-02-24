#!/usr/bin/env python3
# Purpose: ensure ci.mk targets are documented in docs/development/ci/ci.md.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CI_MK = ROOT / "makefiles" / "ci.mk"
CI_DOC = ROOT / "docs" / "development" / "ci" / "ci.md"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)


def _ci_targets() -> list[str]:
    text = CI_MK.read_text(encoding="utf-8")
    targets = [t for t in TARGET_RE.findall(text) if t.startswith("ci") and not t.startswith("internal/")]
    return sorted(set(targets))


def main() -> int:
    text = CI_DOC.read_text(encoding="utf-8")
    missing = [target for target in _ci_targets() if f"`make {target}`" not in text]
    if missing:
        print("ci target docs coverage check failed", file=sys.stderr)
        for target in missing:
            print(f"- missing docs entry for make target: {target}", file=sys.stderr)
        return 1
    print("ci target docs coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
