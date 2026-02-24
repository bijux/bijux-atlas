from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.enforcement.argparse_policy import check_argparse_policy


def test_argparse_policy_check_passes_repo() -> None:
    repo_root = Path(__file__).resolve().parents[3]
    code, errors = check_argparse_policy(repo_root)
    assert code == 0, errors
