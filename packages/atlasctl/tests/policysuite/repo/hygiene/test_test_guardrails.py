from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.contracts.test_guardrails import (
    check_no_duplicated_coverage_paths,
    check_no_unmarked_test_network,
    check_test_determinism_patterns,
    check_test_taxonomy_layout,
    check_test_write_sandbox,
)


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def test_batch16_test_guardrails_pass() -> None:
    root = _repo_root()
    for fn in (
        check_test_taxonomy_layout,
        check_no_unmarked_test_network,
        check_test_write_sandbox,
        check_test_determinism_patterns,
        check_no_duplicated_coverage_paths,
    ):
        code, errors = fn(root)
        assert code == 0, errors
