"""Centralized subprocess execution helpers."""

from __future__ import annotations

import subprocess
from pathlib import Path


def run(
    cmd: list[str],
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    text: bool = True,
    capture_output: bool = False,
    timeout_seconds: int | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        text=text,
        capture_output=capture_output,
        check=False,
        timeout=timeout_seconds,
    )


def check_output(cmd: list[str], cwd: Path | None = None) -> str:
    return subprocess.check_output(cmd, cwd=cwd, text=True)


__all__ = ["check_output", "run"]
