from __future__ import annotations

import json
from pathlib import Path

from ....core.exec import run as run_cmd

CHECK_ID = "repo.root_shape"
DESCRIPTION = "enforce repository root shape contract from root_whitelist.json"

_WHITELIST = "packages/atlasctl/src/atlasctl/checks/tools/root_shape_whitelist.json"


def _is_git_ignored(repo_root: Path, entry_name: str) -> bool:
    result = run_cmd(["git", "check-ignore", "-q", entry_name], cwd=repo_root)
    return result.returncode == 0


def run(repo_root: Path) -> tuple[int, list[str]]:
    whitelist_path = repo_root / _WHITELIST
    config = json.loads(whitelist_path.read_text(encoding="utf-8"))
    required = set(config.get("required", []))
    allowed = set(config.get("allowed", []))
    compat = set(config.get("compat_shims", []))
    local_noise = set(config.get("local_noise", []))
    all_allowed = required | allowed | compat | local_noise

    seen: set[str] = set()
    local_noise_seen: set[str] = set()
    for entry in sorted(repo_root.iterdir(), key=lambda p: p.name):
        if entry.name == ".git":
            continue
        if _is_git_ignored(repo_root, entry.name) and entry.name not in all_allowed:
            continue
        seen.add(entry.name)
        if entry.name in local_noise:
            local_noise_seen.add(entry.name)

    errors: list[str] = []
    for name in sorted(required - seen):
        errors.append(f"ROOT_SHAPE_MISSING_REQUIRED|entry={name}|expected=required")
    for name in sorted(seen - all_allowed):
        errors.append(f"ROOT_SHAPE_UNEXPECTED_ENTRY|entry={name}|expected=required_or_allowed_or_local_noise")
    # Local noise entries are tolerated and should not fail the root-shape contract.

    return (0 if not errors else 1), errors
