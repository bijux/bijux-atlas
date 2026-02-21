from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.pyproject_contracts import (
    check_console_script_entry,
    check_pyproject_no_duplicate_tool_config,
    check_pyproject_required_blocks,
)


def test_pyproject_contract_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    for fn in (
        check_pyproject_required_blocks,
        check_pyproject_no_duplicate_tool_config,
        check_console_script_entry,
    ):
        code, errors = fn(repo_root)
        assert code == 0, (fn.__name__, errors)
