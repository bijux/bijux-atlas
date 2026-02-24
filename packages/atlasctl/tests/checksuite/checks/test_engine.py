from __future__ import annotations

from pathlib import Path

from atlasctl.checks.adapters import FS
from atlasctl.checks.engine import EngineOptions, create_check_context, resolve_run_id, run_checks, write_evidence_ref
from atlasctl.checks.model import CheckContext, CheckDef, CheckSelector, CheckStatus


def _ctx(tmp_path: Path) -> CheckContext:
    fs = FS(repo_root=tmp_path, allowed_roots=("artifacts/evidence/",))
    return CheckContext(repo_root=tmp_path, fs=fs, env={})


def _check(check_id: str, fn, *, slow: bool = False, tags: tuple[str, ...] = ("repo",)) -> CheckDef:
    return CheckDef(
        check_id=check_id,
        domain="repo",
        description=check_id,
        budget_ms=200,
        fn=fn,
        slow=slow,
        tags=tags,
        owners=("platform",),
        effects=("fs_read",),
    )


def test_engine_deterministic_selection_order(tmp_path: Path) -> None:
    def a(_root: Path):
        return 0, []

    def b(_root: Path):
        return 0, []

    checks = (
        _check("checks_repo_z_last", b),
        _check("checks_repo_a_first", a),
    )
    report = run_checks(checks, CheckSelector(), _ctx(tmp_path), options=EngineOptions(only_fast=False, include_slow=True))
    assert [str(row.id) for row in report.rows] == ["checks_repo_a_first", "checks_repo_z_last"]


def test_engine_fail_fast(tmp_path: Path) -> None:
    def fail(_root: Path):
        return 1, ["nope"]

    def after(_root: Path):
        return 0, []

    checks = (
        _check("checks_repo_a_fail", fail),
        _check("checks_repo_b_after", after),
    )
    report = run_checks(checks, None, _ctx(tmp_path), options=EngineOptions(fail_fast=True, only_fast=False, include_slow=True))
    assert len(report.rows) == 1
    assert report.rows[0].status == CheckStatus.FAIL


def test_engine_effect_denial_is_skip(tmp_path: Path) -> None:
    def needs_network(_root: Path):
        return 0, []

    check = CheckDef(
        check_id="checks_repo_network_probe",
        domain="repo",
        description="network",
        budget_ms=200,
        fn=needs_network,
        tags=("repo",),
        owners=("platform",),
        effects=("network",),
    )
    report = run_checks((check,), None, _ctx(tmp_path), options=EngineOptions(only_fast=False, include_slow=True))
    assert report.rows[0].status == CheckStatus.SKIP


def test_engine_max_failures(tmp_path: Path) -> None:
    def fail(_root: Path):
        return 1, ["bad"]

    checks = (
        _check("checks_repo_a_fail", fail),
        _check("checks_repo_b_fail", fail),
        _check("checks_repo_c_fail", fail),
    )
    report = run_checks(checks, None, _ctx(tmp_path), options=EngineOptions(max_failures=2, only_fast=False, include_slow=True))
    assert len(report.rows) == 2


def test_engine_run_id_and_evidence_ref_helpers() -> None:
    assert resolve_run_id("") == "run-deterministic"
    ev = write_evidence_ref(run_id="run-123", rel_path="checks/out.json", content_type="application/json")
    assert ev.path == "artifacts/evidence/run-123/checks/out.json"
    assert ev.content_type == "application/json"


def test_engine_create_context_single_path(tmp_path: Path) -> None:
    ctx = create_check_context(tmp_path)
    assert ctx.repo_root == tmp_path.resolve()
