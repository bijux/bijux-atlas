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
    check_effect_boundary_exceptions_policy,
    check_forbidden_effect_calls,
    check_subprocess_boundary,
)
from atlasctl.checks.repo.enforcement.import_policy import (
    check_checks_import_lint,
    check_command_import_lint,
)
from atlasctl.checks.repo.enforcement.structure.check_structure import (
    check_check_impl_no_cli_imports,
    check_checks_canonical_location,
)
from atlasctl.checks.repo.enforcement.package_shape import check_checks_domain_split
from atlasctl.commands.policies.runtime.culprits import check_critical_dir_count_trend


def _repo_root() -> Path:
    path = Path(__file__).resolve()
    for parent in path.parents:
        if (parent / ".git").exists():
            return parent
    raise AssertionError("unable to locate repository root")


def test_import_boundary_checks_pass() -> None:
    root = _repo_root()
    for fn in (
        check_command_import_lint,
        check_checks_import_lint,
        check_checks_canonical_location,
        check_check_impl_no_cli_imports,
        check_checks_domain_split,
    ):
        code, errors = fn(root)
        assert code == 0, errors


def test_effect_boundary_checks_pass() -> None:
    root = _repo_root()
    for fn in (check_subprocess_boundary, check_forbidden_effect_calls, check_effect_boundary_exceptions_policy):
        code, errors = fn(root)
        assert code == 0, errors


def test_command_inventory_invariant_checks_pass() -> None:
    root = _repo_root()
    for fn in (check_internal_commands_not_public, check_command_alias_budget, check_command_ownership_docs):
        code, errors = fn(root)
        assert code == 0, errors


def test_test_guardrail_checks_pass() -> None:
    # checks_repo_json_goldens_validate_schema
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


def test_critical_dir_count_trend_gate_passes() -> None:
    code, errors = check_critical_dir_count_trend(_repo_root())
    assert code == 0, errors
