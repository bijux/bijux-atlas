from __future__ import annotations

from pathlib import Path

from atlasctl.core.runtime.paths import find_repo_root
from atlasctl.reporting import script_output_dir


def test_repo_root_has_makefile() -> None:
    root = find_repo_root()
    assert (root / "Makefile").exists()


def test_artifacts_scripts_dir_shape() -> None:
    out = script_output_dir("example", "run-1")
    assert Path("artifacts/scripts/example/run-1") == out.relative_to(find_repo_root())


def test_reporting_output_dir() -> None:
    out = script_output_dir("unit-test", "run-2")
    assert out.exists()
    assert "artifacts/scripts/unit-test/run-2" in out.as_posix()
