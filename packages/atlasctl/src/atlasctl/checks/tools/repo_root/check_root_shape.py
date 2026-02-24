from __future__ import annotations

from pathlib import Path

from ....core.exec import run as run_cmd
from ..root_policy import load_root_policy

CHECK_ID = "repo.root_shape"
DESCRIPTION = "enforce repository root shape contract from root_policy.json"


def _is_git_ignored(repo_root: Path, entry_name: str) -> bool:
    result = run_cmd(["git", "check-ignore", "-q", entry_name], cwd=repo_root)
    return result.returncode == 0


def run(repo_root: Path) -> tuple[int, list[str]]:
    policy = load_root_policy(repo_root)

    seen: set[str] = set()
    for entry in sorted(repo_root.iterdir(), key=lambda p: p.name):
        if entry.name == ".git":
            continue
        if _is_git_ignored(repo_root, entry.name) and entry.name not in policy.all_allowed:
            continue
        seen.add(entry.name)

    errors: list[str] = []
    for name in sorted(policy.required - seen):
        errors.append(f"ROOT_SHAPE_MISSING_REQUIRED|entry={name}|expected=required")
    for name in sorted(seen - policy.all_allowed):
        errors.append(f"ROOT_SHAPE_UNEXPECTED_ENTRY|entry={name}|expected=required_or_allowed_or_local_noise")
    # Local noise entries are tolerated and should not fail the root-shape contract.

    return (0 if not errors else 1), errors
