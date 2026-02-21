#!/usr/bin/env python3
# Purpose: enforce wrapper-only contract and target budget for makefiles/ops.mk.
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS_MK = ROOT / "makefiles" / "ops.mk"
ATLASCTL_RE = re.compile(r"^\t@\.\/bin\/atlasctl\s+ops\b")
TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):(?:\s|$)")
MAX_TARGETS = 15


def main() -> int:
    text = OPS_MK.read_text(encoding="utf-8")
    errors: list[str] = []
    targets: list[str] = []

    for lineno, line in enumerate(text.splitlines(), start=1):
        match = TARGET_RE.match(line)
        if match and not line.startswith("."):
            targets.append(match.group(1))

        if not line.startswith("\t"):
            continue

        if "$(MAKE)" in line or " make " in f" {line} ":
            errors.append(f"makefiles/ops.mk:{lineno}: make recursion is forbidden in wrapper-only ops.mk")
        if not ATLASCTL_RE.match(line):
            errors.append(f"makefiles/ops.mk:{lineno}: recipe must delegate with ./bin/atlasctl ops ...")

    if len(targets) > MAX_TARGETS:
        errors.append(
            f"makefiles/ops.mk: target budget exceeded {len(targets)} > {MAX_TARGETS}; collapse wrappers behind atlasctl ops subcommands"
        )

    if errors:
        print("ops.mk contract check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print(f"ops.mk contract check passed: {len(targets)}/{MAX_TARGETS} targets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
