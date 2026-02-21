from __future__ import annotations

import json

from helpers import run_atlasctl


def test_test_smoke_help() -> None:
    proc = run_atlasctl("test", "smoke", "--help")
    assert proc.returncode == 0, proc.stderr
    assert "run fast CLI smoke unit tests" in proc.stdout


def test_test_smoke_json_payload() -> None:
    proc = run_atlasctl("--quiet", "test", "smoke", "--json", "--", "--maxfail=1")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout.splitlines()[0])
    assert payload["suite"] == "smoke"
    assert payload["tool"] == "atlasctl"
