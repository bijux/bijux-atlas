from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.pyproject_contracts import (
    check_deps_workflow_doc,
    check_console_script_entry,
    check_env_docs_present,
    check_optional_dependency_groups,
    check_pyproject_minimalism,
    check_pyproject_no_duplicate_tool_config,
    check_pyproject_required_blocks,
    check_python_module_help,
)


def test_pyproject_contract_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    for fn in (
        check_pyproject_required_blocks,
        check_pyproject_no_duplicate_tool_config,
        check_console_script_entry,
        check_python_module_help,
        check_optional_dependency_groups,
        check_pyproject_minimalism,
        check_deps_workflow_doc,
        check_env_docs_present,
    ):
        code, errors = fn(repo_root)
        assert code == 0, (fn.__name__, errors)
