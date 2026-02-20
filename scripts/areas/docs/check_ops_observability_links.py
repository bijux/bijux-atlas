#!/usr/bin/env python3
# owner: docs-governance
# purpose: validate markdown links in docs/operations/observability/*.md resolve locally.
# stability: public
# called-by: make docs
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOC_DIR = ROOT / "docs/operations/observability"
LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def main() -> int:
    errors: list[str] = []
    for md in sorted(DOC_DIR.glob("*.md")):
        text = md.read_text(encoding="utf-8")
        for link in LINK_RE.findall(text):
            if link.startswith(("http://", "https://", "mailto:", "#")):
                continue
            target = link.split("#", 1)[0]
            if not target:
                continue
            resolved = (md.parent / target).resolve()
            if not resolved.exists():
                errors.append(f"{md.relative_to(ROOT)} -> missing link target: {link}")
    if errors:
        print("ops observability link-check failed:", file=sys.stderr)
        for err in errors:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("ops observability link-check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
