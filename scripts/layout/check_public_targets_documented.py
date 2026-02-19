#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

from public_make_targets import public_names

ROOT = Path(__file__).resolve().parents[2]
README = ROOT / "makefiles" / "README.md"
LINE_RE = re.compile(r"`make\s+([A-Za-z0-9_./-]+)(?:\s+[^`]*)?`")


def main() -> int:
    text = README.read_text(encoding="utf-8")
    documented = set(LINE_RE.findall(text))
    missing = [target for target in public_names() if target not in documented]

    if missing:
        print("public target docs completeness check failed", file=sys.stderr)
        for target in missing:
            print(f"- makefiles/README.md missing line for public target: {target}", file=sys.stderr)
        return 1

    print("public target docs completeness check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
