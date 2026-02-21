from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.pyproject_contracts import (
    check_dependency_gate_targets,
    check_deps_command_surface,
    check_deps_workflow_doc,
    check_dependency_owner_justification,
    check_console_script_entry,
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


def test_pyproject_contract_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    for fn in (
        check_pyproject_required_blocks,
        check_pyproject_no_duplicate_tool_config,
        check_console_script_entry,
        check_python_module_help,
        check_version_matches_pyproject,
        check_packaging_metadata_completeness,
        check_optional_dependency_groups,
        check_pyproject_minimalism,
        check_deps_workflow_doc,
        check_env_docs_present,
        check_requirements_artifact_policy,
        check_requirements_sync_with_pyproject,
        check_dependency_owner_justification,
        check_dependency_gate_targets,
        check_deps_command_surface,
    ):
        code, errors = fn(repo_root)
        assert code == 0, (fn.__name__, errors)
