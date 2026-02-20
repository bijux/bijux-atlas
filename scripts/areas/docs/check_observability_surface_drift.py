#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
TARGET = ROOT / "docs/_generated/observability-surface.md"
GENERATOR = ROOT / "scripts/areas/docs/generate_observability_surface.py"


def main() -> int:
    before = TARGET.read_text(encoding="utf-8") if TARGET.exists() else ""
    subprocess.run([sys.executable, str(GENERATOR)], cwd=ROOT, check=True)
    after = TARGET.read_text(encoding="utf-8") if TARGET.exists() else ""
    if before != after:
        print("observability surface drift detected; regenerate generated docs", file=sys.stderr)
        return 1
    print("observability surface drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
