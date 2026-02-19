#!/usr/bin/env python3
# owner: docs-governance
# purpose: propose durable replacements for legacy temporal/task wording.
# stability: public
# called-by: manual
from __future__ import annotations

import re
import sys
from pathlib import Path

REPLACEMENTS: list[tuple[re.Pattern[str], str]] = [
    (re.compile(r"\\bphase\\s+([0-9]+)\\b", re.IGNORECASE), r"stability level: provisional"),
    (re.compile(r"\\bstep\\s+([0-9]+)\\b", re.IGNORECASE), r"checkpoint"),
    (re.compile(r"\\bstage\\s+([0-9]+)\\b", re.IGNORECASE), r"boundary"),
    (re.compile(r"\\btask\\s+([0-9]+)\\b", re.IGNORECASE), r"requirement"),
    (re.compile(r"\\biteration\\s+([0-9]+)\\b", re.IGNORECASE), r"revision"),
    (re.compile(r"\\bround\\s+([0-9]+)\\b", re.IGNORECASE), r"review cycle"),
    (re.compile(r"\\bWIP\\b", re.IGNORECASE), "draft"),
    (re.compile(r"\\btemporary\\b", re.IGNORECASE), "provisional"),  # ATLAS-EXC-0102: replacement rule must match forbidden legacy token.
    (re.compile(r"vnext\\s+placeholder", re.IGNORECASE), "future extension (documented non-goal)"),
]


def suggest(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    out = text
    for pattern, replacement in REPLACEMENTS:
        out = pattern.sub(replacement, out)
    if out == text:
        return 0
    print(f"--- {path}")
    for idx, (old, new) in enumerate(zip(text.splitlines(), out.splitlines()), start=1):
        if old != new:
            print(f"L{idx}: - {old}")
            print(f"L{idx}: + {new}")
    return 1


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: rewrite_legacy_terms.py <file> [file ...]", file=sys.stderr)
        return 2
    changes = 0
    for arg in sys.argv[1:]:
        p = Path(arg)
        if not p.exists():
            print(f"missing: {p}", file=sys.stderr)
            return 2
        changes += suggest(p)
    return 0 if changes == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
