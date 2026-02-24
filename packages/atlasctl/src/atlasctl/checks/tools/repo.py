from __future__ import annotations

from .repo_domain import CHECKS, layout
from .repo_domain.contracts.pyproject_contracts import (
    check_pyproject_no_duplicate_tool_config,
    check_pyproject_required_blocks,
)
from .repo_domain.enforcement.package_shape import check_module_size
from .repo_domain.enforcement.package_shape import check_atlasctl_package_root_shape
from .repo_domain.native import (
    check_docs_no_ops_generated_run_paths,
    check_docs_scripts_references,
    check_forbidden_top_dirs,
    check_python_lock,
    check_python_migration_exceptions_expiry,
)
from .repo_domain.native.modules.repo_checks_make_and_layout import check_layout_contract
from .repo_domain.root_determinism import DESCRIPTION as ROOT_DETERMINISM_DESCRIPTION
from .repo_domain.root_determinism import run as run_root_determinism
from .repo_domain.root_forbidden_files import DESCRIPTION as FORBIDDEN_ROOT_FILES_DESCRIPTION
from .repo_domain.root_forbidden_files import run as run_forbidden_root_files
from .repo_domain.root_forbidden_names import DESCRIPTION as FORBIDDEN_ROOT_NAMES_DESCRIPTION
from .repo_domain.root_forbidden_names import run as run_forbidden_root_names
from .repo_domain.root_forbidden_paths import DESCRIPTION as FORBIDDEN_PATHS_DESCRIPTION
from .repo_domain.root_forbidden_paths import run as run_forbidden_paths
from .repo_domain.root_shape import DESCRIPTION as ROOT_SHAPE_DESCRIPTION
from .repo_domain.root_shape import run as run_root_shape

__all__ = [
    "CHECKS",
    "layout",
    "FORBIDDEN_PATHS_DESCRIPTION",
    "FORBIDDEN_ROOT_FILES_DESCRIPTION",
    "FORBIDDEN_ROOT_NAMES_DESCRIPTION",
    "ROOT_DETERMINISM_DESCRIPTION",
    "ROOT_SHAPE_DESCRIPTION",
    "check_atlasctl_package_root_shape",
    "check_docs_no_ops_generated_run_paths",
    "check_docs_scripts_references",
    "check_forbidden_top_dirs",
    "check_layout_contract",
    "check_module_size",
    "check_pyproject_no_duplicate_tool_config",
    "check_pyproject_required_blocks",
    "check_python_lock",
    "check_python_migration_exceptions_expiry",
    "run_forbidden_paths",
    "run_forbidden_root_files",
    "run_forbidden_root_names",
    "run_root_determinism",
    "run_root_shape",
]
