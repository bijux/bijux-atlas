#!/usr/bin/env python3
# Purpose: run shellcheck against ops shell scripts with stable excludes.
# Inputs: ops/**/*.sh scripts.
# Outputs: shellcheck diagnostics and exit code.
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def _ops_shell_files(root: Path) -> list[Path]:
    return sorted((root / "ops").rglob("*.sh"))


def _run_local_shellcheck(files: list[Path], shellcheck_rc: Path) -> int:
    cmd = ["shellcheck", "--rcfile", str(shellcheck_rc), "-x", *[str(p) for p in files]]
    return subprocess.run(cmd, cwd=_repo_root()).returncode


def _run_docker_shellcheck(files: list[Path], root: Path) -> int:
    print("shellcheck not found locally; using docker image")
    for f in files:
        rel = f.relative_to(root)
        rc = subprocess.run(
            [
                "docker",
                "run",
                "--rm",
                "-v",
                f"{root}:/mnt",
                "koalaman/shellcheck:stable",
                "--rcfile",
                "/mnt/configs/shellcheck/shellcheckrc",
                "-x",
                f"/mnt/{rel.as_posix()}",
            ]
        ).returncode
        if rc != 0:
            return rc
    return 0


def main() -> int:
    root = _repo_root()
    shellcheck_rc = root / "configs/shellcheck/shellcheckrc"
    files = _ops_shell_files(root)
    if not files:
        print("no ops shell scripts found")
        return 0

    if shutil.which("shellcheck"):
        return _run_local_shellcheck(files, shellcheck_rc)

    if shutil.which("docker"):
        rc = _run_docker_shellcheck(files, root)
        if rc == 0:
            return 0

    if os.environ.get("SHELLCHECK_STRICT", "0") == "1":
        print("shellcheck is required (install shellcheck or docker)", file=sys.stderr)
        return 1

    print("shellcheck skipped: shellcheck/docker unavailable (non-strict mode)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
