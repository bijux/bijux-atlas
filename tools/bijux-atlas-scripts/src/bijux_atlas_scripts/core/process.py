from __future__ import annotations

import subprocess
import time
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class CommandResult:
    code: int
    stdout: str
    stderr: str
    duration_ms: int

    @property
    def combined_output(self) -> str:
        return (self.stdout + self.stderr).strip()


def run_command(cmd: list[str], cwd: Path) -> CommandResult:
    started = time.monotonic()
    proc = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    duration_ms = int((time.monotonic() - started) * 1000)
    return CommandResult(
        code=proc.returncode,
        stdout=proc.stdout or "",
        stderr=proc.stderr or "",
        duration_ms=duration_ms,
    )
