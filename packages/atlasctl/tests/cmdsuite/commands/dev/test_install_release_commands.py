from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_install_doctor_json() -> None:
    proc = run_atlasctl("--quiet", "install", "doctor", "--json")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["run_id"]
    assert isinstance(payload["tools"], list)


def test_release_checklist_dry_run_json() -> None:
    proc = run_atlasctl("--quiet", "release", "checklist", "--plan", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "release-checklist"
    assert payload["plan"] is True
    ids = [row["id"] for row in payload["checks"]]
    assert "suite_release_0_1" in ids
