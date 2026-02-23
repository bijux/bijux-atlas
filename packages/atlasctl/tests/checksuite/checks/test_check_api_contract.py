from __future__ import annotations

from pathlib import Path

from atlasctl.checks.api import check
from atlasctl.checks.core.base import CheckDef
from atlasctl.checks.engine.execution import run_function_checks


def test_check_decorator_attaches_metadata() -> None:
    @check(check_id="checks_repo_demo_contract", domain="repo", intent="validate demo contract")
    def demo(repo_root: Path) -> tuple[int, list[str]]:
        return 0, []

    meta = getattr(demo, "__atlasctl_check_meta__", None)
    assert meta is not None
    assert meta.check_id == "checks_repo_demo_contract"
    assert meta.domain == "repo"


def test_runtime_guard_forbids_print_calls(tmp_path: Path) -> None:
    def noisy(_repo_root: Path) -> tuple[int, list[str]]:
        print("not allowed")
        return 0, []

    check_def = CheckDef("checks_repo_noisy", "repo", "noisy", 1000, noisy)
    failed, rows = run_function_checks(tmp_path, [check_def], run_root=tmp_path / "artifacts" / "evidence")
    assert failed == 1
    assert rows[0].status == "fail"
    assert any("must not print" in msg for msg in rows[0].errors)


def test_runtime_guard_blocks_writes_without_effect(tmp_path: Path) -> None:
    target = tmp_path / "forbidden.txt"

    def writer(_repo_root: Path) -> tuple[int, list[str]]:
        target.write_text("x", encoding="utf-8")
        return 0, []

    check_def = CheckDef("checks_repo_writer", "repo", "writer", 1000, writer)
    failed, rows = run_function_checks(tmp_path, [check_def], run_root=tmp_path / "artifacts" / "evidence")
    assert failed == 1
    assert rows[0].status == "fail"
    assert any("effects.write=true" in msg or "internal check error" in msg for msg in rows[0].errors)
