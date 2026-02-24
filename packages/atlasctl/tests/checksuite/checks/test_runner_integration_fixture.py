from __future__ import annotations

from pathlib import Path

from atlasctl.checks.adapters import FS
from atlasctl.checks.model import CheckContext, CheckDef
from atlasctl.checks.policy import Capabilities
from atlasctl.checks.runner import run_checks


def _repo_file_exists(repo_root: Path) -> tuple[int, list[str]]:
    target = repo_root / "README.md"
    if target.exists():
        return 0, []
    return 1, [f"missing file: {target.relative_to(repo_root).as_posix()}"]


def _repo_forbidden_file_absent(repo_root: Path) -> tuple[int, list[str]]:
    target = repo_root / "forbidden.txt"
    if target.exists():
        return 1, [f"forbidden file present: {target.relative_to(repo_root).as_posix()}"]
    return 0, []


def test_runner_executes_subset_against_temp_repo_fixture(monkeypatch, tmp_path: Path) -> None:
    (tmp_path / "README.md").write_text("fixture\n", encoding="utf-8")
    checks = (
        CheckDef("checks_repo_fixture_readme_exists", "repo", "fixture readme exists", 200, _repo_file_exists, owners=("platform",)),
        CheckDef("checks_repo_fixture_forbidden_absent", "repo", "fixture forbidden file absent", 200, _repo_forbidden_file_absent, owners=("platform",)),
    )
    monkeypatch.setattr("atlasctl.checks.runner.list_checks", lambda: checks)

    ctx = CheckContext(repo_root=tmp_path, fs=FS(repo_root=tmp_path, allowed_roots=("artifacts/evidence",)))
    report = run_checks(ctx, capabilities=Capabilities(allow_network=False))

    assert [row.id for row in report.rows] == [
        "checks_repo_fixture_forbidden_absent",
        "checks_repo_fixture_readme_exists",
    ]
    assert [row.status.value for row in report.rows] == ["pass", "pass"]
