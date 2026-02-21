#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
TARGET = ROOT / "ops/_meta/surface.json"
GEN = ROOT / "packages/atlasctl/src/atlasctl/checks/layout/ops/generation/generate_ops_surface_meta.py"


def main() -> int:
    before = TARGET.read_text(encoding="utf-8") if TARGET.exists() else ""
    subprocess.run([sys.executable, str(GEN)], cwd=ROOT, check=True)
    after = TARGET.read_text(encoding="utf-8") if TARGET.exists() else ""
    if before != after:
        print("ops surface metadata drift detected; regenerate ops/_meta/surface.json", file=sys.stderr)
        return 1
    print("ops surface metadata drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
