#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
ALLOWLIST = ROOT / "configs" / "layout" / "python-legacy-allowlist.txt"


def load_allowlist() -> set[str]:
    if not ALLOWLIST.exists():
        return set()
    return {
        line.strip()
        for line in ALLOWLIST.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.strip().startswith("#")
    }


def main() -> int:
    allow = load_allowlist()
    errs: list[str] = []
    proc = subprocess.run(
        ["git", "ls-files", "*.py"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    for rel in sorted(ln.strip() for ln in proc.stdout.splitlines() if ln.strip()):
        if rel.startswith("tools/bijux-atlas-scripts/"):
            continue
        if rel in allow:
            continue
        errs.append(rel)
    if errs:
        print("no ad-hoc python script check failed", file=sys.stderr)
        for item in errs[:200]:
            print(f"- unregistered python file outside tools package: {item}", file=sys.stderr)
        return 1
    print("no ad-hoc python script check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
