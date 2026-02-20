#!/usr/bin/env python3
from __future__ import annotations

import fnmatch
import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ALLOWED_PREFIX = (ROOT / "artifacts").resolve()


def _allowed(path: Path) -> bool:
    resolved = path.resolve()
    return resolved == ALLOWED_PREFIX or ALLOWED_PREFIX in resolved.parents


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fix", action="store_true", help="remove forbidden runtime artifacts in-place")
    ns = parser.parse_args()
    violations: list[str] = []
    paths_to_remove: list[Path] = []

    # Forbidden runtime dirs outside artifacts.
    forbidden_dirs = {".venv", ".ruff_cache", ".pytest_cache", ".mypy_cache", "__pycache__", ".hypothesis"}
    for p in ROOT.rglob("*"):
        if p.is_dir() and p.name in forbidden_dirs and not _allowed(p):
            if ".git" in p.parts:
                continue
            violations.append(f"forbidden dir outside artifacts: {p.relative_to(ROOT)}")
            paths_to_remove.append(p)

    # Any .pyc outside artifacts is forbidden.
    for p in ROOT.rglob("*.pyc"):
        if not _allowed(p):
            if ".git" in p.parts:
                continue
            violations.append(f"forbidden pyc outside artifacts: {p.relative_to(ROOT)}")
            paths_to_remove.append(p)

    # Tracked pyc must never exist.
    tracked = subprocess.run(
        ["git", "ls-files"],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    for rel in tracked.stdout.splitlines():
        if fnmatch.fnmatch(rel, "*.pyc"):
            violations.append(f"tracked pyc file: {rel}")

    if violations:
        if ns.fix:
            for path in sorted(set(paths_to_remove), key=lambda p: len(p.parts), reverse=True):
                if path.is_dir():
                    subprocess.run(["rm", "-rf", str(path)], check=False)
                elif path.is_file():
                    path.unlink(missing_ok=True)
            print(f"python runtime artifact policy auto-fixed ({len(paths_to_remove)} paths)")
            return 0
        print("python runtime artifact policy failed:", file=sys.stderr)
        for item in violations[:200]:
            print(f"- {item}", file=sys.stderr)
        return 1

    print("python runtime artifact policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
