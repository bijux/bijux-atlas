from __future__ import annotations

from pathlib import Path
from typing import Any

from .culprits import biggest_dirs, biggest_files, check_budget_metric


def check_py_files_per_dir(repo_root: Path) -> tuple[int, list[str]]:
    return check_budget_metric(repo_root, "py-files-per-dir")


def check_modules_per_dir(repo_root: Path) -> tuple[int, list[str]]:
    return check_budget_metric(repo_root, "modules-per-dir")


def check_loc_per_dir(repo_root: Path) -> tuple[int, list[str]]:
    return check_budget_metric(repo_root, "dir-loc")


def worst_directories_by_loc(repo_root: Path, *, limit: int = 20) -> dict[str, Any]:
    return biggest_dirs(repo_root, limit=limit)


def worst_files_by_loc(repo_root: Path, *, limit: int = 20) -> dict[str, Any]:
    return biggest_files(repo_root, limit=limit)
