from __future__ import annotations

import os
from pathlib import Path

from .....core.process import run_command


REGISTRY_PATH = "packages/atlasctl/src/atlasctl/checks/REGISTRY.toml"
REQUIRED_OWNERS = "makefiles/ownership.json"
REQUIRED_DOCS = {"docs/checks/registry.md", "docs/INDEX.md"}
REQUIRED_GOLDENS = {
    "packages/atlasctl/tests/goldens/check/check-list.json.golden",
    "packages/atlasctl/tests/goldens/check/checks-tree.json.golden",
    "packages/atlasctl/tests/goldens/check/checks-owners.json.golden",
}


def _changed_files(repo_root: Path) -> set[str]:
    proc = run_command(
        ["git", "diff", "--name-only", "HEAD"],
        cwd=repo_root,
    )
    if proc.code != 0:
        return set()
    return {line.strip() for line in (proc.stdout or "").splitlines() if line.strip()}


def _gate(repo_root: Path, *, required: set[str], label: str) -> tuple[int, list[str]]:
    if str(os.environ.get("CI", "")).lower() not in {"1", "true", "yes"}:
        return 0, []
    changed = _changed_files(repo_root)
    if REGISTRY_PATH not in changed:
        return 0, []
    if any(path in changed for path in required):
        return 0, []
    needed = ", ".join(sorted(required))
    return 1, [f"registry changed; require {label} update: {needed}"]


def check_registry_change_requires_owner_update(repo_root: Path) -> tuple[int, list[str]]:
    return _gate(repo_root, required={REQUIRED_OWNERS}, label="owners")


def check_registry_change_requires_docs_update(repo_root: Path) -> tuple[int, list[str]]:
    return _gate(repo_root, required=REQUIRED_DOCS, label="docs index")


def check_registry_change_requires_golden_update(repo_root: Path) -> tuple[int, list[str]]:
    return _gate(repo_root, required=REQUIRED_GOLDENS, label="goldens")
