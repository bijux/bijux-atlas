from __future__ import annotations

import json

from tests.helpers import run_atlasctl


def test_registry_select_checks_by_domain_and_severity_json() -> None:
    proc = run_atlasctl("--quiet", "registry", "select", "checks", "--domain", "repo", "--severity", "error", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert "checks" in payload
    assert all(item.startswith("checks_repo_") for item in payload["checks"])


def test_registry_select_commands_by_group_text() -> None:
    proc = run_atlasctl("--quiet", "registry", "select", "commands", "--group", "check")
    assert proc.returncode == 0, proc.stderr
    rows = [ln.strip() for ln in proc.stdout.splitlines() if ln.strip()]
    assert "check" in rows


def test_registry_diff_runs_check_mode() -> None:
    proc = run_atlasctl("--quiet", "registry", "diff")
    assert proc.returncode in {0, 2, 20}, proc.stderr
