#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CFG = ROOT / "configs/scripts/python-tooling.json"


def main() -> int:
    cfg = json.loads(CFG.read_text(encoding="utf-8"))
    if cfg.get("toolchain") != "pip-tools":
        raise SystemExit("python tooling SSOT must declare toolchain=pip-tools")
    lockfile = ROOT / str(cfg["lockfile"])
    if not lockfile.exists():
        raise SystemExit(f"missing lockfile: {lockfile}")

    pyproject = ROOT / "tools/bijux-atlas-scripts/pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    m = re.search(r"\[project\.optional-dependencies\]\s*dev\s*=\s*\[(?P<body>.*?)\]", text, re.S)
    if not m:
        raise SystemExit("unable to parse [project.optional-dependencies].dev from pyproject.toml")
    expected = sorted(re.findall(r'"([^"]+)"', m.group("body")))

    lines = [ln.strip() for ln in lockfile.read_text(encoding="utf-8").splitlines() if ln.strip() and not ln.startswith("#")]
    locked = sorted(lines)
    if expected != locked:
        print("scripts lock drift detected:")
        print(f"- expected (pyproject dev): {expected}")
        print(f"- locked: {locked}")
        return 1
    print("scripts lock check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
