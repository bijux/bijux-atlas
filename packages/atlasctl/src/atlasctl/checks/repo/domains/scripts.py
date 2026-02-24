"""Repository script-surface checks."""

from __future__ import annotations

from ..native.modules.repo_checks_scripts_and_docker import (
    check_docs_scripts_references,
    check_make_scripts_references,
)
from ..native.runtime_modules.repo_native_runtime_core import (
    check_duplicate_script_names,
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
