from __future__ import annotations

from pathlib import Path

from ....core.exec import run as run_cmd

CHECK_ID = "repo.no_direct_script_runs"
DESCRIPTION = "forbid direct scripts/ or ops/ invocations in GitHub workflows"

_PATTERN = r"run:\s*\.?/?(scripts|ops)/"
_WORKFLOWS_DIR = ".github/workflows"


def run(repo_root: Path) -> tuple[int, list[str]]:
    proc = run_cmd(
        ["rg", "-n", _PATTERN, _WORKFLOWS_DIR],
        cwd=repo_root,
        text=True,
        capture_output=True,
    )
    if proc.returncode == 0:
        lines = [line for line in (proc.stdout or "").splitlines() if line.strip()]
        return 1, lines
    if proc.returncode == 1:
        return 0, []
    message = (proc.stderr or proc.stdout or "workflow direct-script scan failed").strip()
    return 1, [message]
