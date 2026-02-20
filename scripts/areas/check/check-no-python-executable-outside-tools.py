#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys

from python_migration_exceptions import find_matching_exception

PACKAGE_PREFIX = "packages/bijux-atlas-scripts/"


def main() -> int:
    proc = subprocess.run(
        ["git", "ls-files", "--stage", "*.py"],
        check=True,
        capture_output=True,
        text=True,
    )
    errors: list[str] = []
    for line in proc.stdout.splitlines():
        if not line.strip():
            continue
        mode, _obj, _stage_and_path = line.split(maxsplit=2)
        _stage, rel = _stage_and_path.split("\t", 1)
        if mode != "100755":
            continue
        if rel.startswith(PACKAGE_PREFIX):
            continue
        if "/tests/" in rel:
            continue
        if find_matching_exception("executable_python", rel, rel) is not None:
            continue
        errors.append(rel)

    if errors:
        print("python executable placement check failed:", file=sys.stderr)
        for rel in errors:
            print(f"- executable .py outside scripts package: {rel}", file=sys.stderr)
        return 1

    print("python executable placement check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
