from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_test_smoke_help() -> None:
    proc = run_atlasctl("test", "smoke", "--help")
    assert proc.returncode == 0, proc.stderr
    assert "emit machine-readable summary" in proc.stdout


def test_test_smoke_json_payload() -> None:
    proc = run_atlasctl("--quiet", "test", "smoke", "--json", "--", "--maxfail=1")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout.splitlines()[0])
    assert payload["suite"] == "smoke"
    assert payload["tool"] == "atlasctl"


def test_test_run_unit_json_payload(tmp_path) -> None:
    target = tmp_path / "isolate"
    proc = run_atlasctl("--quiet", "test", "run", "unit", "--json", "--target-dir", str(target))
    assert proc.returncode in (0, 1, 2), proc.stderr
    payload = json.loads(proc.stdout.splitlines()[0])
    assert payload["suite"] == "unit"
    assert payload["tool"] == "atlasctl"
    assert payload["target_dir"] == str(target)
