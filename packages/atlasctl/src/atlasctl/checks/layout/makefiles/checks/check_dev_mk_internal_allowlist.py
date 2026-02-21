#!/usr/bin/env python3
# Purpose: keep dev.mk internal targets restricted to explicit allowlist.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
DEV_MK = ROOT / "makefiles" / "dev.mk"
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)")
ALLOWED_INTERNAL = {"internal/dev/check"}


def main() -> int:
    errors: list[str] = []
    for lineno, line in enumerate(DEV_MK.read_text(encoding="utf-8").splitlines(), start=1):
        match = TARGET_RE.match(line)
        if not match or line.startswith("."):
            continue
        target = match.group(1)
        if target.startswith("internal/") and target not in ALLOWED_INTERNAL:
            errors.append(
                f"makefiles/dev.mk:{lineno}: internal target `{target}` is forbidden; allowed={sorted(ALLOWED_INTERNAL)}"
            )
    if errors:
        print("dev.mk internal allowlist check failed", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("dev.mk internal allowlist check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
