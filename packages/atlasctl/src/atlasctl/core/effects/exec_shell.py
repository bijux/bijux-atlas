from __future__ import annotations

from pathlib import Path

from ..process import run_command
from ..runtime.repo_root import find_repo_root


def run_shell_script(script: Path, args: list[str] | None = None, cwd: Path | None = None) -> dict[str, object]:
    resolved = script.resolve()
    resolved_cwd = cwd.resolve() if cwd else find_repo_root()
    cmd = ["bash", str(resolved), *(args or [])]
    proc = run_command(cmd, cwd=resolved_cwd)
    return {
        "command": cmd,
        "script": str(resolved),
        "cwd": str(resolved_cwd),
        "exit_code": proc.code,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "status": "ok" if proc.code == 0 else "error",
    }
