from __future__ import annotations

from pathlib import Path

from atlasctl.checks.repo.contracts.command_contracts import (
    check_command_alias_budget,
    check_command_ownership_docs,
    check_internal_commands_not_public,
)
from atlasctl.checks.repo.contracts.test_guardrails import (
    check_check_test_coverage,
    check_command_test_coverage,
    check_json_goldens_validate_schema,
    check_suite_marker_rules,
    check_test_duplicate_expectations,
    check_test_ownership_tags,
)
from atlasctl.checks.repo.enforcement.boundaries.effect_boundaries import (
    check_forbidden_effect_calls,
    check_subprocess_boundary,
)
from atlasctl.checks.repo.enforcement.import_policy import (
    check_checks_import_lint,
    check_command_import_lint,
)


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def test_import_boundary_checks_pass() -> None:
    root = _repo_root()
    for fn in (check_command_import_lint, check_checks_import_lint):
        code, errors = fn(root)
        assert code == 0, errors


def test_effect_boundary_checks_pass() -> None:
    root = _repo_root()
    for fn in (check_subprocess_boundary, check_forbidden_effect_calls):
        code, errors = fn(root)
        assert code == 0, errors


def test_command_inventory_invariant_checks_pass() -> None:
    root = _repo_root()
    for fn in (check_internal_commands_not_public, check_command_alias_budget, check_command_ownership_docs):
        code, errors = fn(root)
        assert code == 0, errors


def test_test_guardrail_checks_pass() -> None:
    # repo.json_goldens_validate_schema
    root = _repo_root()
    for fn in (
        check_test_duplicate_expectations,
        check_test_ownership_tags,
        check_suite_marker_rules,
        check_command_test_coverage,
        check_json_goldens_validate_schema,
        check_check_test_coverage,
    ):
        code, errors = fn(root)
        assert code == 0, errors
