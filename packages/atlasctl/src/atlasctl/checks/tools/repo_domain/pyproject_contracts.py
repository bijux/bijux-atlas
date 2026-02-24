from __future__ import annotations

from .contracts.pyproject_contracts import (
    check_console_script_entry,
    check_dependency_gate_targets,
    check_dependency_owner_justification,
    check_deps_command_surface,
    check_deps_workflow_doc,
    check_env_docs_present,
    check_optional_dependency_groups,
    check_packaging_metadata_completeness,
    check_pyproject_minimalism,
    check_pyproject_no_duplicate_tool_config,
    check_pyproject_required_blocks,
    check_python_module_help,
    check_requirements_artifact_policy,
    check_requirements_sync_with_pyproject,
    check_version_matches_pyproject,
)

__all__ = [
    "check_dependency_gate_targets",
    "check_dependency_owner_justification",
    "check_deps_command_surface",
    "check_deps_workflow_doc",
    "check_console_script_entry",
    "check_env_docs_present",
    "check_optional_dependency_groups",
    "check_packaging_metadata_completeness",
    "check_pyproject_minimalism",
    "check_pyproject_required_blocks",
    "check_pyproject_no_duplicate_tool_config",
    "check_python_module_help",
    "check_requirements_artifact_policy",
    "check_requirements_sync_with_pyproject",
    "check_version_matches_pyproject",
]
