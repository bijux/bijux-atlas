#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

_THIS_DIR = Path(__file__).resolve().parent
if str(_THIS_DIR) not in sys.path:
    sys.path.insert(0, str(_THIS_DIR))

from public_make_targets import public_names

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for base in (cur, *cur.parents):
        if (base / "makefiles").exists() and (base / "packages").exists():
            return base
    raise RuntimeError("unable to resolve repository root")


ROOT = _repo_root()
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

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
