#!/usr/bin/env python3
# Purpose: enforce root Makefile hygiene (size and shelling rules).
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAKEFILE = ROOT / "Makefile"
MAX_LINES = 80


def main() -> int:
    text = MAKEFILE.read_text(encoding="utf-8")
    lines = text.splitlines()
    errs: list[str] = []

    if len(lines) > MAX_LINES:
        errs.append(f"Makefile too large: {len(lines)} lines (max {MAX_LINES})")

    stripped = [ln.strip() for ln in lines if ln.strip() and not ln.strip().startswith("#")]
    allowed = {"SHELL := /bin/sh", "include makefiles/root.mk"}
    for ln in stripped:
        if ln in allowed:
            continue
        errs.append(f"Makefile must not shell out or define commands directly: {ln}")

    if errs:
        print("root Makefile hygiene check failed", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("root Makefile hygiene check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
