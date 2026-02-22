from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_dev_help_snapshot() -> None:
    proc = run_atlasctl("--quiet", "dev", "--help")
    assert proc.returncode == 0, proc.stderr
    lines = [line.rstrip() for line in proc.stdout.splitlines() if line.strip()]
    assert lines[0] == "usage: atlasctl dev [-h] [--list] [--json] [--verbose]"
    assert "{list,suite,ci,commands,explain,fmt,lint,check,test,coverage,audit,split-module}" in proc.stdout


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
