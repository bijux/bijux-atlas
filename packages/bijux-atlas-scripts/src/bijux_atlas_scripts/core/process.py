from __future__ import annotations

import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

from .logging import log_event

if TYPE_CHECKING:
    from .context import RunContext


@dataclass(frozen=True)
class CommandResult:
    code: int
    stdout: str
    stderr: str
    duration_ms: int

    @property
    def combined_output(self) -> str:
        return (self.stdout + self.stderr).strip()


def run_command(
    cmd: list[str],
    cwd: Path,
    timeout_seconds: int = 0,
    retries: int = 0,
    retry_delay_seconds: float = 0.0,
    ctx: RunContext | None = None,
) -> CommandResult:
    attempt = 0
    last_result: CommandResult | None = None
    while attempt <= retries:
        attempt += 1
        started = time.monotonic()
        try:
            proc = subprocess.run(
                cmd,
                cwd=cwd,
                text=True,
                capture_output=True,
                check=False,
                timeout=(timeout_seconds if timeout_seconds > 0 else None),
            )
            result = CommandResult(
                code=proc.returncode,
                stdout=proc.stdout or "",
                stderr=proc.stderr or "",
                duration_ms=int((time.monotonic() - started) * 1000),
            )
        except subprocess.TimeoutExpired as exc:
            result = CommandResult(
                code=124,
                stdout=(exc.stdout or ""),
                stderr=((exc.stderr or "") + f"\ncommand timed out after {timeout_seconds}s").strip(),
                duration_ms=int((time.monotonic() - started) * 1000),
            )
        if ctx and not ctx.quiet:
            log_event(
                ctx,
                "info",
                "process",
                "run-command",
                command=" ".join(cmd),
                cwd=str(cwd),
                attempt=attempt,
                code=result.code,
                duration_ms=result.duration_ms,
            )
        last_result = result
        if result.code == 0 or attempt > retries:
            return result
        if retry_delay_seconds > 0:
            time.sleep(retry_delay_seconds)
    return last_result if last_result is not None else CommandResult(1, "", "unknown error", 0)
