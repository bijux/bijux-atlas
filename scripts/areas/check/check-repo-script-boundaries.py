#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from python_migration_exceptions import find_matching_exception

ROOT = Path(__file__).resolve().parents[3]


def _tracked(pattern: str) -> list[str]:
    proc = subprocess.run(
        ["git", "ls-files", pattern],
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    )
    return sorted([ln.strip() for ln in proc.stdout.splitlines() if ln.strip()])


def main() -> int:
    errors: list[str] = []

    # 42: no scripts/ dir allowed (unless transitional exception exists)
    scripts_files = [p for p in _tracked("scripts/**") if not p.endswith(".md")]
    for rel in scripts_files:
        if find_matching_exception("scripts_dir", rel, "") is None:
            errors.append(f"scripts directory transition is closed; file must move under packages/: {rel}")

    # 43: no executable .py outside packages/
    exec_proc = subprocess.run(
        ["git", "ls-files", "--stage", "*.py"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    for line in exec_proc.stdout.splitlines():
        if not line.strip():
            continue
        mode, _obj, stage_path = line.split(maxsplit=2)
        _stage, rel = stage_path.split("\t", 1)
        if mode != "100755":
            continue
        if rel.startswith("packages/") or rel.startswith("tools/"):
            continue
        if "/tests/" in rel:
            continue
        if find_matching_exception("executable_python", rel, rel) is None:
            errors.append(f"executable python outside packages/: {rel}")

    # 44: shell scripts only under docker/ or packages/.../resources (plus transition exception)
    for rel in _tracked("*.sh"):
        if rel.startswith("docker/") or rel.startswith("packages/"):
            continue
        if find_matching_exception("shell_location", rel, "") is None:
            errors.append(f"shell script outside docker/ or packages/: {rel}")

    if errors:
        print("repo script boundary check failed:", file=sys.stderr)
        for err in errors[:200]:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("repo script boundary check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
