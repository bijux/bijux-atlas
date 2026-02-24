from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.contracts.dependencies import check_dependency_declarations
from atlasctl.checks.tools.reachability import check_repo_check_modules_registered
from atlasctl.checks.repo.contracts.type_coverage import check_type_coverage


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[6]


def test_type_coverage_check_passes_current_repo() -> None:
    code, errors = check_type_coverage(_repo_root())
    assert code == 0
    assert errors == []


def test_dependency_declaration_check_passes_current_repo() -> None:
    code, errors = check_dependency_declarations(_repo_root())
    assert code == 0
    assert errors == []


def test_repo_check_reachability_check_passes_current_repo() -> None:
    code, errors = check_repo_check_modules_registered(_repo_root())
    assert code == 0
    assert errors == []
