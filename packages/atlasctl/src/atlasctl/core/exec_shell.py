from __future__ import annotations

from pathlib import Path

from .exec import run


def run_shell_script(script: Path, args: list[str] | None = None, cwd: Path | None = None) -> dict[str, object]:
    resolved = script.resolve()
    cmd = ["bash", str(resolved), *(args or [])]
    proc = run(cmd, cwd=cwd, text=True, capture_output=True)
    return {
        "command": cmd,
        "script": str(resolved),
        "cwd": str(cwd.resolve() if cwd else Path.cwd().resolve()),
        "exit_code": proc.returncode,
        "stdout": proc.stdout or "",
        "stderr": proc.stderr or "",
        "status": "ok" if proc.returncode == 0 else "error",
    }
