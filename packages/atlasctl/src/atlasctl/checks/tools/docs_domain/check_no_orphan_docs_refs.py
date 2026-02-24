#!/usr/bin/env python3
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
DOCS = ROOT / "docs"
MAKE_TARGET_RE = re.compile(r"`make\s+([A-Za-z0-9_./-]+)\b[^`]*`")


def load_make_targets() -> set[str]:
    proc = subprocess.run(["make", "-qp"], cwd=ROOT, text=True, capture_output=True, check=False)
    targets: set[str] = set()
    for line in proc.stdout.splitlines():
        if ":" not in line or line.startswith("\t") or line.startswith("#"):
            continue
        name = line.split(":", 1)[0].strip()
        if not name or any(ch in name for ch in " %$()"):
            continue
        targets.add(name)
    return targets


def main() -> int:
    targets = load_make_targets()
    errors: list[str] = []
    for md in sorted(p for p in DOCS.rglob("*.md") if "docs/_generated/" not in p.as_posix()):
        rel = md.relative_to(ROOT).as_posix()
        text = md.read_text(encoding="utf-8", errors="ignore")
        for i, line in enumerate(text.splitlines(), start=1):
            for m in MAKE_TARGET_RE.finditer(line):
                target = m.group(1)
                if target not in targets:
                    errors.append(f"{rel}:{i}: documented make target does not exist: `{target}`")

    if errors:
        print("orphan docs command references check failed:", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
