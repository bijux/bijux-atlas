from __future__ import annotations

from .legacy_native import (
    check_docs_no_ops_generated_run_paths,
    check_duplicate_script_names,
    check_forbidden_top_dirs,
    check_no_executable_python_outside_packages,
    check_no_ops_generated_placeholder,
    check_no_xtask_refs,
    check_ops_examples_immutable,
    check_ops_generated_tracked,
    check_tracked_timestamp_paths,
)
from ..base import CheckDef
from .legacy_guard import check_legacy_package_quarantine
from .module_size import check_module_size
from .cwd_usage import check_no_path_cwd_usage
from .scripts_dir import check_scripts_dir_absent

CHECKS: tuple[CheckDef, ...] = (
    CheckDef("repo/forbidden-top-dirs", "repo", 500, check_forbidden_top_dirs),
    CheckDef("repo/no-docs-ops-generated-run-refs", "repo", 800, check_docs_no_ops_generated_run_paths),
    CheckDef("repo/no-xtask-refs", "repo", 1000, check_no_xtask_refs),
    CheckDef("repo/no-exec-python-outside-packages", "repo", 1500, check_no_executable_python_outside_packages),
    CheckDef("repo/no-tracked-ops-generated", "repo", 1000, check_ops_generated_tracked),
    CheckDef("repo/no-ops-generated-placeholder", "repo", 400, check_no_ops_generated_placeholder),
    CheckDef("repo/ops-examples-immutable", "repo", 800, check_ops_examples_immutable),
    CheckDef("repo/no-tracked-timestamp-paths", "repo", 1000, check_tracked_timestamp_paths),
    CheckDef("repo/duplicate-script-names", "repo", 1200, check_duplicate_script_names),
    CheckDef("repo/no-scripts-dir", "repo", 250, check_scripts_dir_absent),
    CheckDef("repo/legacy-quarantine", "repo", 250, check_legacy_package_quarantine),
    CheckDef("repo/module-size", "repo", 400, check_module_size),
    CheckDef("repo/no-path-cwd-usage", "repo", 400, check_no_path_cwd_usage),
)
