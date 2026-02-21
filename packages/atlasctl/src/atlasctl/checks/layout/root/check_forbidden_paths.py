from __future__ import annotations

from pathlib import Path

from ....core.exec import run as run_cmd

CHECK_ID = "repo.no_forbidden_paths"
DESCRIPTION = "forbid legacy root path references in tracked text surfaces"

_TARGETS = ("Makefile", "makefiles", ".github", "ops", "scripts")
_PATTERN = r"\./(charts|e2e|load|observability|datasets|fixtures)/|docs/operations/ops/"
_SELF_PATH = "packages/atlasctl/src/atlasctl/checks/layout/root/check_forbidden_paths.py"


def run(repo_root: Path) -> tuple[int, list[str]]:
    cmd = ["rg", "-n", _PATTERN, *_TARGETS, "-g", f"!{_SELF_PATH}"]
    proc = run_cmd(cmd, cwd=repo_root, text=True, capture_output=True)
    if proc.returncode == 0:
        lines = [line for line in (proc.stdout or "").splitlines() if line.strip()]
        return 1, lines
    if proc.returncode == 1:
        return 0, []
    message = (proc.stderr or proc.stdout or "forbidden path scan failed").strip()
    return 1, [message]
