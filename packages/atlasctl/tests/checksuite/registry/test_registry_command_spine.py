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


def test_registry_select_invalid_subject_has_well_defined_error() -> None:
    proc = run_atlasctl("--quiet", "registry", "select", "bad-subject")
    assert proc.returncode != 0
    assert "invalid choice" in (proc.stderr or proc.stdout)


def test_registry_diff_runs_check_mode() -> None:
    proc = run_atlasctl("--quiet", "registry", "diff")
    assert proc.returncode in {0, 2, 20}, proc.stderr


def test_registry_validate_runs() -> None:
    proc = run_atlasctl("--quiet", "registry", "validate")
    assert proc.returncode in {0, 2}, proc.stderr


def test_registry_gate_runs() -> None:
    proc = run_atlasctl("--quiet", "registry", "gate")
    assert proc.returncode in {0, 2, 20}, proc.stderr


def test_registry_rename_check_id_dry_run_runs() -> None:
    proc = run_atlasctl("--quiet", "registry", "rename-check-id", "--json")
    assert proc.returncode in {0, 1, 2}, proc.stderr
