"""Python runtime/policy adapter helpers."""

from __future__ import annotations

import sys
from pathlib import Path


__all__ = ["detect_venv", "python_lockfile_exists", "runtime_policy"]


def detect_venv() -> bool:
    return hasattr(sys, "base_prefix") and sys.prefix != getattr(sys, "base_prefix", sys.prefix)


def python_lockfile_exists(repo_root: Path) -> bool:
    return (repo_root / "packages/atlasctl/requirements.lock.txt").is_file()


def runtime_policy(repo_root: Path) -> dict[str, object]:
    return {
        "python_version": sys.version.split()[0],
        "venv": detect_venv(),
        "python_lockfile": python_lockfile_exists(repo_root),
    }
