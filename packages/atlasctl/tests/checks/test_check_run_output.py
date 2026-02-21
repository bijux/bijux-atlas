from __future__ import annotations

import json
from pathlib import Path

from helpers import run_atlasctl_isolated

ROOT = Path(__file__).resolve().parents[1]


def _golden(name: str) -> str:
    return (ROOT / "goldens" / name).read_text(encoding="utf-8").strip()


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
        "--junitxml",
        str(junit_report),
    )
    assert proc.returncode == 0, proc.stderr
    assert json_report.exists()
    assert junit_report.exists()
    payload = json.loads(json_report.read_text(encoding="utf-8"))
    assert payload["kind"] == "check-run-report"
    assert payload["summary"]["failed"] == 0
    assert payload["summary"]["total"] >= 1


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
