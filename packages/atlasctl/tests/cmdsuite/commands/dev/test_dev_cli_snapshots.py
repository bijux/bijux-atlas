from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_dev_help_snapshot() -> None:
    proc = run_atlasctl("--quiet", "dev", "--help")
    assert proc.returncode == 0, proc.stderr
    lines = [line.rstrip() for line in proc.stdout.splitlines() if line.strip()]
    assert lines[0] == "usage: atlasctl dev [-h] [--list] [--json] [--verbose]"
    assert "clean" in proc.stdout
    assert "doctor" in proc.stdout
    assert "split-module" in proc.stdout


def test_dev_ci_help_snapshot() -> None:
    proc = run_atlasctl("--quiet", "dev", "ci", "--help")
    assert proc.returncode == 0, proc.stderr
    lines = [line.rstrip() for line in proc.stdout.splitlines() if line.strip()]
    assert lines[0] == "usage: atlasctl dev ci [-h] ..."
    assert "positional arguments:" in proc.stdout


def test_dev_ci_run_explain_snapshot() -> None:
    proc = run_atlasctl("--quiet", "--json", "--run-id", "snapshot-ci-explain", "dev", "ci", "run", "--explain")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["action"] == "ci-run-explain"
    assert payload["run_id"] == "pytest-run"
    assert payload["planned_steps"][0]["id"] == "ci.step.001"


def test_dev_doctor_json_reports_cache_hygiene() -> None:
    proc = run_atlasctl("--quiet", "--json", "dev", "doctor")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["command"] == "doctor"
    assert "cache_hygiene" in payload


def test_dev_clean_json_snapshot() -> None:
    proc = run_atlasctl("--quiet", "--json", "dev", "clean")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["status"] == "ok"
