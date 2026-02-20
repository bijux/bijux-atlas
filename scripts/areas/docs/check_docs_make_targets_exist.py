#!/usr/bin/env python3
# Purpose: ensure every `make <target>` reference in docs points to an existing make target.
# Inputs: docs/**/*.md and `make -qp`.
# Outputs: non-zero exit when docs reference missing targets.
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
DOCS = ROOT / "docs"

LINE_CMD_RE = re.compile(r"^\s*(?:\$|#)?\s*(?:[A-Za-z_][A-Za-z0-9_]*=[^\s]+\s+)*make\s+([A-Za-z0-9_./-]+)")
INLINE_CMD_RE = re.compile(r"`(?:[A-Za-z_][A-Za-z0-9_]*=[^\s`]+\s+)*make\s+([A-Za-z0-9_./-]+)`")


def load_targets() -> set[str]:
    out = subprocess.run(["make", "-qp"], cwd=ROOT, text=True, capture_output=True, check=False)
    targets: set[str] = set()
    for line in out.stdout.splitlines():
        if ":" not in line or line.startswith("\t") or line.startswith("#"):
            continue
        candidate = line.split(":", 1)[0].strip()
        if not candidate:
            continue
        if any(ch in candidate for ch in " %$()"):
            continue
        targets.add(candidate)
    return targets


def main() -> int:
    targets = load_targets()
    missing: list[str] = []
    for path in sorted(DOCS.rglob("*.md")):
        rel = path.relative_to(ROOT)
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            matches = []
            matches.extend(LINE_CMD_RE.finditer(line))
            matches.extend(INLINE_CMD_RE.finditer(line))
            for match in matches:
                target = match.group(1)
                if target not in targets:
                    missing.append(f"{rel}:{lineno}: unknown make target `{target}`")
    if missing:
        print("docs make-target existence check failed:", file=sys.stderr)
        for item in missing[:200]:
            print(f"- {item}", file=sys.stderr)
        return 1
    print("docs make-target existence check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
