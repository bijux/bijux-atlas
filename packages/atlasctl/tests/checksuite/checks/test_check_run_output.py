from __future__ import annotations

import json
from pathlib import Path

from atlasctl.contracts.validate import validate
from tests.helpers import golden_text, run_atlasctl_isolated


def _golden(name: str) -> str:
    return golden_text(name)


def test_check_run_quiet_output_golden(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--select",
        "atlasctl::docs::__no_match__",
        "--quiet",
    )
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check-run-quiet.txt.golden")


def test_check_run_info_output_golden(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--select",
        "atlasctl::docs::__no_match__",
        "--info",
    )
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check-run-info.txt.golden")


def test_check_run_verbose_output_golden(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--select",
        "atlasctl::docs::__no_match__",
        "--verbose",
    )
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip() == _golden("check-run-verbose.txt.golden")


def test_check_run_writes_json_and_junit_reports(tmp_path: Path) -> None:
    evidence_root = tmp_path / "evidence"
    json_report = evidence_root / "reports" / "check-run.json"
    junit_report = evidence_root / "reports" / "check-run.xml"
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--select",
        "atlasctl::docs::__no_match__",
        "--maxfail",
        "2",
        "--failfast",
        "--durations",
        "2",
        "--json-report",
        str(json_report),
        "--junit-xml",
        str(junit_report),
    )
    assert proc.returncode == 0, proc.stderr
    assert json_report.exists()
    assert junit_report.exists()
    payload = json.loads(json_report.read_text(encoding="utf-8"))
    validate("atlasctl.check-run.v1", payload)
    assert payload["kind"] == "check-run-report"
    assert payload["summary"]["failed"] == 0
    assert payload["summary"]["total"] >= 0


def test_check_run_profile_and_slow_report(tmp_path: Path) -> None:
    evidence_root = tmp_path / "evidence"
    slow_report = evidence_root / "reports" / "check-slow.json"
    profile_out = evidence_root / "reports" / "check-profile.json"
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--select",
        "atlasctl::docs::__no_match__",
        "--profile",
        "--profile-out",
        str(profile_out),
        "--slow-threshold-ms",
        "1",
        "--slow-report",
        str(slow_report),
    )
    assert proc.returncode == 0, proc.stderr
    assert slow_report.exists()
    assert profile_out.exists()


def test_check_run_accepts_check_target_and_jsonl(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "atlasctl::docs::__no_match__",
        "--jsonl",
    )
    assert proc.returncode == 0, proc.stderr
    lines = [line for line in proc.stdout.splitlines() if line.strip()]
    assert any('"kind": "summary"' in line for line in lines)


def test_check_show_source_and_unknown_exit_code(tmp_path: Path) -> None:
    known = run_atlasctl_isolated(tmp_path, "--quiet", "check", "--show-source", "checks_repo_module_size")
    assert known.returncode == 0, known.stderr
    assert known.stdout.strip().endswith("checks/repo/enforcement/package_shape.py")

    unknown = run_atlasctl_isolated(tmp_path, "--quiet", "check", "--show-source", "repo.__missing__")
    assert unknown.returncode == 2, unknown.stderr


def test_check_run_group_and_match_filters(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--group",
        "docs",
        "--match",
        "atlasctl::docs::*",
        "--require-markers",
        "docs",
        "--select",
        "atlasctl::docs::__no_match__",
        "--quiet",
    )
    assert proc.returncode == 0, proc.stderr
    assert proc.stdout.strip()


def test_check_run_json_quiet_validates_schema(tmp_path: Path) -> None:
    proc = run_atlasctl_isolated(
        tmp_path,
        "--quiet",
        "check",
        "run",
        "--group",
        "repo",
        "--select",
        "atlasctl::repo::__no_match__",
        "--json",
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    validate("atlasctl.check-run.v1", payload)
    assert payload["schema_name"] == "atlasctl.check-run.v1"
