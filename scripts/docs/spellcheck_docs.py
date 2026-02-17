#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
import shutil
import subprocess
import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: spellcheck_docs.py <docs_dir>", file=sys.stderr)
        return 2
    root = Path(sys.argv[1])
    targets = [root / "index.md", root / "_style"]

    exe = shutil.which("codespell")
    if not exe:
        print("codespell not found in PATH", file=sys.stderr)
        return 2
    cmd = [
        exe,
        "--quiet-level",
        "2",
        "--skip",
        "*.json,*.png,*.jpg,*.svg",
    ]

    for target in targets:
        if target.exists():
            cmd.append(str(target))

    proc = subprocess.run(cmd, check=False)
    if proc.returncode != 0:
        print("spellcheck failed", file=sys.stderr)
        return proc.returncode
    print("spellcheck passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())