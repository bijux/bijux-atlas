"""Compatibility shim for `atlasctl.commands.policies.runtime.dir_budgets`."""

from .budgets.dir_budgets import (
    check_loc_per_dir,
    check_modules_per_dir,
    check_py_files_per_dir,
    worst_directories_by_loc,
    worst_files_by_loc,
)

__all__ = [
    "check_loc_per_dir",
    "check_modules_per_dir",
    "check_py_files_per_dir",
    "worst_directories_by_loc",
    "worst_files_by_loc",
]
