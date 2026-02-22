from __future__ import annotations

import re
from pathlib import Path

from ....core.process import run_command

_TARGET_RE = re.compile(r"^([A-Za-z0-9_./-]+):")
_WORKFLOW_MAKE_RE = re.compile(r"\bmake(?:\s+-[A-Za-z0-9_-]+)*\s+([A-Za-z0-9_./-]+)")


def _make_targets(repo_root: Path) -> set[str]:
    result = run_command(["make", "-qp"], cwd=repo_root)
    targets: set[str] = set()
    for line in result.stdout.splitlines():
        match = _TARGET_RE.match(line)
        if not match:
            continue
        name = match.group(1).strip()
        if not name or name.startswith(".") or "%" in name:
            continue
        targets.add(name)
    return targets


def check_workflows_targets_exist(repo_root: Path) -> tuple[int, list[str]]:
    known_targets = _make_targets(repo_root)
    if not known_targets:
        return 1, ["failed to enumerate make targets via `make -qp`"]
    errors: list[str] = []
    workflows_root = repo_root / ".github" / "workflows"
    for workflow in sorted(workflows_root.glob("*.yml")):
        text = workflow.read_text(encoding="utf-8", errors="ignore")
        for line_no, line in enumerate(text.splitlines(), start=1):
            for match in _WORKFLOW_MAKE_RE.finditer(line):
                target = match.group(1)
                if target not in known_targets:
                    errors.append(f"{workflow.relative_to(repo_root).as_posix()}:{line_no}: missing make target `{target}`")
    return (0 if not errors else 1), sorted(errors)
