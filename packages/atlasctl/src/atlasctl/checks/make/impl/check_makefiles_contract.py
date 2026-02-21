#!/usr/bin/env python3
# Purpose: validate makefiles contract boundaries and publication rules.
# Inputs: makefiles/*.mk, make -pn output, makefiles/CONTRACT.md.
# Outputs: non-zero on contract violations.
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
SURFACE = ROOT / "configs/ops/public-surface.json"
CONTRACT = ROOT / "makefiles/CONTRACT.md"

TARGET_RE = re.compile(r"^([a-zA-Z0-9_.-]+):(?:\s|$)", re.M)


def parse_targets(path: Path) -> set[str]:
    return set(t for t in TARGET_RE.findall(path.read_text(encoding="utf-8")) if not t.startswith("."))


def main() -> int:
    errs: list[str] = []
    if not CONTRACT.exists():
        errs.append("missing makefiles/CONTRACT.md")

    surface = json.loads(SURFACE.read_text(encoding="utf-8"))
    public_targets = set(surface.get("make_targets", [])) - {"help"}

    root_text = (ROOT / "makefiles/root.mk").read_text(encoding="utf-8")
    root_phony: set[str] = set()
    for line in root_text.splitlines():
        if line.startswith(".PHONY:"):
            root_phony.update(line.replace(".PHONY:", "", 1).split())
    for t in sorted(public_targets):
        if t not in root_phony:
            errs.append(f"public target missing from makefiles/root.mk .PHONY publication surface: {t}")

    proc = subprocess.run(["make", "-pn"], cwd=ROOT, text=True, capture_output=True)
    if proc.returncode != 0:
        errs.append("make -pn failed")
    else:
        if "makefiles/root.mk" not in proc.stdout:
            errs.append("make -pn output missing root.mk reference")

    if errs:
        print("makefiles contract check failed", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("makefiles contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
