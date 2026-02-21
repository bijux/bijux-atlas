from __future__ import annotations

from pathlib import Path

from atlasctl.contracts.json import read_json, write_json
from atlasctl.internal.system import dump_env, run_timed
from atlasctl.paths.artifacts import artifacts_scripts_dir


def test_json_helpers_roundtrip(tmp_path: Path) -> None:
    path = tmp_path / "payload.json"
    payload = {"b": 2, "a": 1}
    write_json(path, payload)
    assert read_json(path) == payload


def test_artifacts_scripts_dir_shape_migration_helpers(monkeypatch, tmp_path: Path) -> None:
    (tmp_path / ".git").mkdir()
    monkeypatch.chdir(tmp_path)
    out = artifacts_scripts_dir("example", "run-1")
    assert out.as_posix().endswith("/artifacts/scripts/example/run-1")


def test_internal_helpers_dump_and_exec(monkeypatch, tmp_path: Path) -> None:
    (tmp_path / ".git").mkdir()
    monkeypatch.chdir(tmp_path)

    env_path = dump_env(script_name="env_dump_test", run_id="r1")
    assert env_path.exists()
    assert "run_id=r1" in env_path.read_text(encoding="utf-8")

    code, timing_path = run_timed(["sh", "-c", "true"], script_name="exec_test", run_id="r2")
    assert code == 0
    assert timing_path.exists()
