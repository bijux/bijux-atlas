from __future__ import annotations

from pathlib import Path

from atlasctl.checks.adapters import FS
from atlasctl.checks.model import CheckDef
from atlasctl.checks.effects import CheckEffect
from atlasctl.engine.execution import run_function_checks


def test_fs_adapter_enforces_allowed_roots(tmp_path: Path) -> None:
    fs = FS(repo_root=tmp_path, allowed_roots=("artifacts/evidence/",))
    fs.write_text(Path("artifacts/evidence/demo.txt"), "ok")
    try:
        fs.write_text(Path("docs/bad.txt"), "nope")
    except PermissionError:
        pass
    else:  # pragma: no cover
        raise AssertionError("expected PermissionError for disallowed write path")


def test_engine_fails_on_undeclared_subprocess_effect(tmp_path: Path) -> None:
    import subprocess

    def runner(_repo_root: Path) -> tuple[int, list[str]]:
        subprocess.run(["echo", "hi"], check=False, capture_output=True, text=True)
        return 0, []

    check_def = CheckDef(
        "checks_repo_effect_mismatch",
        "repo",
        "effect mismatch",
        1000,
        runner,
        effects=(CheckEffect.FS_READ.value,),
    )
    failed, rows = run_function_checks(tmp_path, [check_def], run_root=tmp_path / "artifacts" / "evidence")
    assert failed == 1
    assert rows[0].status == "fail"
    assert any("effects.subprocess=true" in msg or "undeclared effects used" in msg for msg in rows[0].errors)
