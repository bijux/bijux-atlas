#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[6]
PATTERN = re.compile(r"(?:\./)?ops/(?!run/)[a-z0-9_-]+/scripts/[A-Za-z0-9_./-]+\.sh")


def scan_file(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    errs: list[str] = []
    for m in PATTERN.finditer(text):
        lineno = text.count("\n", 0, m.start()) + 1
        errs.append(f"{path.relative_to(ROOT)}:{lineno}: external reference must use ops/run/* -> `{m.group(0)}`")
    return errs


def main() -> int:
    errs: list[str] = []
    scan_paths = [
        ROOT / "docs",
        ROOT / ".github/workflows",
        ROOT / "makefiles/root.mk",
        ROOT / "makefiles/ci.mk",
    ]
    for base in scan_paths:
        if not base.exists():
            continue
        if base.is_file():
            errs.extend(scan_file(base))
            continue
        for path in sorted(base.rglob("*")):
            if path.suffix not in {".md", ".mk", ".sh", ".yml", ".yaml"}:
                continue
            rel = path.relative_to(ROOT).as_posix()
            if rel.startswith("docs/_generated/") or rel.startswith("docs/_lint/"):
                continue
            errs.extend(scan_file(path))

    if errs:
        print("ops external entrypoint check failed:", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1

    print("ops external entrypoint check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
