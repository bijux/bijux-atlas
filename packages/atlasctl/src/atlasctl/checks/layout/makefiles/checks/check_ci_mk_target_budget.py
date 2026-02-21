#!/usr/bin/env python3
# Purpose: cap ci.mk target count to keep CI wrapper surface minimal.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
CI_MK = ROOT / "makefiles" / "ci.mk"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)", re.M)
MAX_TARGETS = 15


def main() -> int:
    text = CI_MK.read_text(encoding="utf-8")
    targets = [t for t in TARGET_RE.findall(text) if not t.startswith(".")]
    total = len(targets)
    if total > MAX_TARGETS:
        print(
            f"ci.mk target budget exceeded: {total} > {MAX_TARGETS} (collapse CI wrappers behind atlasctl ci subcommands)",
            file=sys.stderr,
        )
        return 1
    print(f"ci.mk target budget check passed: {total}/{MAX_TARGETS}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
