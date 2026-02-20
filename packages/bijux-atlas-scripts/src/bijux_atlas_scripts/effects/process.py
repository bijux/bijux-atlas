from __future__ import annotations

from pathlib import Path

from ..core.context import RunContext
from ..core.process import CommandResult, run_command


def run(
    ctx: RunContext,
    cmd: list[str],
    cwd: Path,
    timeout_seconds: int = 0,
    retries: int = 0,
) -> CommandResult:
    return run_command(
        cmd=cmd,
        cwd=cwd,
        timeout_seconds=timeout_seconds,
        retries=retries,
        ctx=ctx,
    )
