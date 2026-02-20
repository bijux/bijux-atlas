"""Repository script-surface checks."""

from __future__ import annotations

from ...check.native import (
    check_docs_scripts_references,
    check_duplicate_script_names,
    check_make_scripts_references,
    check_script_errors,
    check_script_help,
    check_script_ownership,
)

__all__ = [
    "check_docs_scripts_references",
    "check_duplicate_script_names",
    "check_make_scripts_references",
    "check_script_errors",
    "check_script_help",
    "check_script_ownership",
]
