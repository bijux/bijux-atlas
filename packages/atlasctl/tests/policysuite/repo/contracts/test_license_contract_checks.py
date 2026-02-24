from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.policies_domain.licensing.policy import (
    check_license_file_mit,
    check_license_statements_consistent,
    check_spdx_policy,
)


def test_license_contract_checks_pass_repo() -> None:
    repo_root = Path(__file__).resolve().parents[4]
    for fn in (check_license_file_mit, check_license_statements_consistent, check_spdx_policy):
        code, errors = fn(repo_root)
        assert code == 0, (fn.__name__, errors)
