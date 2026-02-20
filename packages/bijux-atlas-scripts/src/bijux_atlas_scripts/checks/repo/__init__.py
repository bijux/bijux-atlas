from __future__ import annotations

from ...check.native import (
    check_duplicate_script_names,
    check_forbidden_top_dirs,
    check_no_executable_python_outside_packages,
    check_no_xtask_refs,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ..base import CheckDef

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("repo/forbidden-top-dirs", "repo", 500, check_forbidden_top_dirs),
    CheckDef("repo/no-xtask-refs", "repo", 1000, check_no_xtask_refs),
    CheckDef("repo/no-exec-python-outside-packages", "repo", 1500, check_no_executable_python_outside_packages),
    CheckDef("repo/no-tracked-ops-generated", "repo", 1000, check_ops_generated_tracked),
    CheckDef("repo/no-tracked-timestamp-paths", "repo", 1000, check_tracked_timestamp_paths),
    CheckDef("repo/duplicate-script-names", "repo", 1200, check_duplicate_script_names),
)
