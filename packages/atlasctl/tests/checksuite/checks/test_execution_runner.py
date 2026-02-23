from __future__ import annotations

from pathlib import Path

from atlasctl.execution.runner import RunnerOptions, run_checks_payload
from atlasctl.checks.core.base import CheckDef


def _ok(_root: Path) -> tuple[int, list[str]]:
    return 0, []


def _fail(_root: Path) -> tuple[int, list[str]]:
    return 1, ["E001 bad thing", "WARN: cleanup needed"]


def test_runner_emits_check_run_payload_with_events_findings_and_attachments(tmp_path: Path) -> None:
    check = CheckDef(
        check_id="checks_repo_runner_sample",
        domain="repo",
        description="sample",
        budget_ms=100,
        fn=_fail,
        fix_hint="fix it",
        owners=("platform",),
        evidence=("artifacts/evidence/sample.txt",),
    )
    rc, payload = run_checks_payload(tmp_path, check_defs=[check], run_id="t1", options=RunnerOptions(run_root=tmp_path))
    assert rc == 1
    assert payload["schema_name"] == "atlasctl.check-run.v1"
    assert payload["rows"][0]["findings"]
    assert "events" in payload
    assert "attachments" in payload
    assert payload["rows"][0]["category"] in {"check", "lint"}


def test_runner_dry_run_returns_ordered_skip_rows(tmp_path: Path) -> None:
    a = CheckDef("checks_repo_a", "repo", "a", 100, _ok)
    b = CheckDef("checks_repo_b", "repo", "b", 100, _ok)
    rc, payload = run_checks_payload(tmp_path, check_defs=[b, a], run_id="t2", options=RunnerOptions(dry_run=True))
    assert rc == 0
    assert [row["id"] for row in payload["rows"]] == ["checks_repo_a", "checks_repo_b"]
    assert all(row["status"] == "SKIP" for row in payload["rows"])
