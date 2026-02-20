#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REQUIRED = {
    "makefiles/targets.json",
    "docs/_generated/make-targets.md",
}


def changed_files() -> set[str]:
    files: set[str] = set()
    cmds = [
        ["git", "diff", "--name-only"],
        ["git", "diff", "--name-only", "--cached"],
        ["git", "diff", "--name-only", "HEAD~1", "HEAD"],
    ]
    for cmd in cmds:
        proc = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=False)
        if proc.returncode != 0:
            continue
        for line in proc.stdout.splitlines():
            if line.strip():
                files.add(line.strip())
    return files


def main() -> int:
    changed = changed_files()
    if "makefiles/root.mk" not in changed:
        print("root diff alarm check passed")
        return 0

    missing = sorted(r for r in REQUIRED if r not in changed)
    if missing:
        # If companion files are unchanged, allow only when regeneration is a no-op.
        regen = subprocess.run(
            ["python3", "-m", "atlasctl.cli", "docs", "generate-make-targets-catalog", "--report", "text"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        if regen.returncode == 0:
            drift = subprocess.run(
                ["git", "diff", "--name-only", "--", "makefiles/targets.json", "docs/_generated/make-targets.md"],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            if drift.returncode == 0 and not drift.stdout.strip():
                print("root diff alarm check passed")
                return 0
        print("root diff alarm check failed", file=sys.stderr)
        print("- makefiles/root.mk changed without inventory/docs updates", file=sys.stderr)
        for m in missing:
            print(f"- missing companion change: {m}", file=sys.stderr)
        return 1

    print("root diff alarm check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
