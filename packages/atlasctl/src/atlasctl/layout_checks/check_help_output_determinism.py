#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CMD = ["python3", "./packages/atlasctl/src/atlasctl/layout_checks/render_public_help.py"]


def main() -> int:
    a = subprocess.check_output(CMD, cwd=ROOT, text=True)
    b = subprocess.check_output(CMD, cwd=ROOT, text=True)
    if a != b:
        print("help output determinism check failed", file=sys.stderr)
        return 1
    print("help output determinism check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
