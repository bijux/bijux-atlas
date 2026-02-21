from __future__ import annotations

from pathlib import Path

from ..process import run_command


def run_shell_script(script: Path, args: list[str] | None = None, cwd: Path | None = None) -> dict[str, object]:
    resolved = script.resolve()
    cmd = ["bash", str(resolved), *(args or [])]
    proc = run_command(cmd, cwd=(cwd or Path.cwd()))
    return {
        "command": cmd,
        "script": str(resolved),
        "cwd": str(cwd.resolve() if cwd else Path.cwd().resolve()),
        "exit_code": proc.code,
        "stdout": proc.stdout,
        "stderr": proc.stderr,
        "status": "ok" if proc.code == 0 else "error",
    }
