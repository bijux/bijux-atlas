#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOC = ROOT / "docs/development/tooling/bijux-atlas-scripts.md"
CFG = ROOT / "configs/scripts/python-tooling.json"


def main() -> int:
    payload = json.loads(CFG.read_text(encoding="utf-8"))
    commands = [str(cmd) for cmd in payload.get("commands", [])]
    text = DOC.read_text(encoding="utf-8")
    missing = [cmd for cmd in commands if f"`{cmd}`" not in text]
    if missing:
        print("scripts command surface docs drift detected:", file=sys.stderr)
        for cmd in missing:
            print(f"- missing `{cmd}` in {DOC.relative_to(ROOT)}", file=sys.stderr)
        return 1
    print("scripts command surface docs drift check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
