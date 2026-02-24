from __future__ import annotations

from pathlib import Path

from atlasctl.checks.tools.repo_domain.contracts.public_api import check_public_api_exports


def test_public_api_exports_check_passes_repo() -> None:
    repo_root = Path(__file__).resolve().parents[4]
    code, errors = check_public_api_exports(repo_root)
    assert code == 0
    assert errors == []
