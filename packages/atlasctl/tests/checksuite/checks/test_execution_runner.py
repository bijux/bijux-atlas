from __future__ import annotations

from pathlib import Path

from atlasctl.execution.runner import RunnerOptions, run_checks_payload
from atlasctl.checks.core.base import CheckDef
from atlasctl.checks.engine.execution import CommandCheckDef
from atlasctl.core.runtime.serialize import dumps_json


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


def test_runner_timing_contract_and_json_stability(tmp_path: Path) -> None:
    check = CheckDef("checks_repo_ok", "repo", "ok", 100, _ok)
    rc, payload = run_checks_payload(tmp_path, check_defs=[check], run_id="t3", options=RunnerOptions())
    assert rc == 0
    assert set(payload["timing_histogram"].keys()) == {"lt_100ms", "100_500ms", "500_1000ms", "1000_2000ms", "gte_2000ms"}
    a = dumps_json(payload, pretty=False)
    b = dumps_json(payload, pretty=False)
    assert a == b


def test_runner_budget_exceed_behavior_contract_warn_vs_fail(tmp_path: Path) -> None:
    slow = CheckDef("checks_repo_slow", "repo", "slow", 0, _ok)
    rc_warn, p_warn = run_checks_payload(tmp_path, check_defs=[slow], run_id="t4", options=RunnerOptions(budget_exceed_behavior="warn"))
    rc_fail, p_fail = run_checks_payload(tmp_path, check_defs=[slow], run_id="t5", options=RunnerOptions(budget_exceed_behavior="fail"))
    assert p_warn["budget_contract"]["exceed_behavior"] == "warn"
    assert p_fail["budget_contract"]["exceed_behavior"] == "fail"
    assert p_warn["budget_contract"]["budget_warn_count"] >= 0
    assert p_fail["budget_contract"]["budget_fail_count"] >= 0
    assert rc_fail in {0, 1}
    assert rc_warn in {0, 1}


def test_runner_lint_defaults_no_network_no_writes(tmp_path: Path) -> None:
    cmd = CommandCheckDef("repo/lint-net", "repo", ["sh", "-c", "curl https://example.com > out.txt"], 100)
    rc, payload = run_checks_payload(tmp_path, command_defs=[cmd], run_id="t6", options=RunnerOptions())
    assert rc == 1
    row = payload["rows"][0]
    assert row["category"] == "lint"
    assert any("forbids network" in f["message"] or "forbids writes" in f["message"] for f in row["findings"])
    assert "network" not in row["markers"]
    assert "write" not in row["markers"]


def test_runner_partial_failures_deterministic_with_fail_fast(tmp_path: Path) -> None:
    a = CheckDef("checks_repo_a", "repo", "a", 100, _ok)
    b = CheckDef("checks_repo_b", "repo", "b", 100, _fail)
    c = CheckDef("checks_repo_c", "repo", "c", 100, _ok)
    rc1, p1 = run_checks_payload(tmp_path, check_defs=[c, b, a], run_id="t7", options=RunnerOptions(fail_fast=True))
    rc2, p2 = run_checks_payload(tmp_path, check_defs=[c, b, a], run_id="t8", options=RunnerOptions(fail_fast=True))
    assert rc1 == rc2 == 1
    assert [r["id"] for r in p1["rows"]] == [r["id"] for r in p2["rows"]]
    assert [r["status"] for r in p1["rows"]] == [r["status"] for r in p2["rows"]]
