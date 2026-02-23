from __future__ import annotations

from pathlib import Path
from types import SimpleNamespace

import pytest

from atlasctl.checks.adapters import FS
from atlasctl.checks.model import CheckContext, CheckDef, CheckId, CheckSelector, DomainId, ResultCode
from atlasctl.checks.policy import Capabilities
from atlasctl.checks.registry import TAGS_VOCAB, list_checks, resolve_aliases
from atlasctl.checks.report import build_report_payload, render_text
from atlasctl.checks.runner import run_checks
from atlasctl.checks.selectors import apply_selection_criteria, parse_selection_criteria, select_checks


def _ok(_repo_root: Path) -> tuple[int, list[str]]:
    return 0, []


def _fail(_repo_root: Path) -> tuple[int, list[str]]:
    return 1, ["boom"]


def test_check_id_parse_validation() -> None:
    assert str(CheckId.parse("checks_repo_parser_contract")) == "checks_repo_parser_contract"
    with pytest.raises(ValueError):
        CheckId.parse("repo.parser.contract")
    with pytest.raises(ValueError):
        CheckId.parse("checks_REPO_upper")
    with pytest.raises(ValueError):
        CheckId.parse("checks_ops_missing_domain", domain="repo")


def test_domain_vocab_enforcement() -> None:
    assert str(DomainId("repo")) == "repo"
    with pytest.raises(ValueError):
        DomainId("unknown-domain")


def test_result_code_validation() -> None:
    assert str(ResultCode("CHECK_GENERIC")) == "CHECK_GENERIC"
    with pytest.raises(ValueError):
        ResultCode("check_generic")


def test_selector_internal_hidden_unless_enabled() -> None:
    checks = [
        CheckDef("checks_repo_visible_contract", "repo", "visible", 100, _ok, tags=("required",), owners=("platform",)),
        CheckDef("checks_repo_internal_contract", "repo", "internal", 100, _ok, tags=("internal",), owners=("platform",)),
    ]
    ns = SimpleNamespace(
        domain_filter="repo",
        id="",
        select="",
        check_target="",
        marker=[],
        tag=[],
        exclude_marker=[],
        exclude_tag=[],
        owner=[],
        include_internal=False,
        only_slow=False,
        only_fast=False,
        changed_only=False,
        k="",
    )
    criteria = parse_selection_criteria(ns, Path(".").resolve())
    selected = apply_selection_criteria(checks, criteria)
    assert [c.check_id for c in selected] == ["checks_repo_visible_contract"]
    ns.include_internal = True
    criteria_all = parse_selection_criteria(ns, Path(".").resolve())
    selected_all = apply_selection_criteria(checks, criteria_all)
    assert [c.check_id for c in selected_all] == ["checks_repo_internal_contract", "checks_repo_visible_contract"]


def test_runner_ordering_fail_fast_and_effect_denied_skip(monkeypatch, tmp_path: Path) -> None:
    checks = (
        CheckDef("checks_repo_alpha_contract", "repo", "alpha", 100, _fail, effects=("fs_read",), owners=("platform",)),
        CheckDef("checks_repo_beta_contract", "repo", "beta", 100, _ok, effects=("network",), owners=("platform",)),
        CheckDef("checks_repo_gamma_contract", "repo", "gamma", 100, _ok, effects=("fs_read",), owners=("platform",)),
    )
    monkeypatch.setattr("atlasctl.checks.runner.list_checks", lambda: checks)
    ctx = CheckContext(repo_root=tmp_path.resolve(), fs=FS(repo_root=tmp_path.resolve(), allowed_roots=("artifacts/evidence",)))
    report = run_checks(ctx, fail_fast=False, capabilities=Capabilities(allow_network=False))
    assert [row.id for row in report.rows] == [
        "checks_repo_alpha_contract",
        "checks_repo_beta_contract",
        "checks_repo_gamma_contract",
    ]
    assert [row.status.value for row in report.rows] == ["fail", "skip", "pass"]
    assert any("effect policy" in msg for msg in report.rows[1].warnings)
    report_fast = run_checks(ctx, fail_fast=True, capabilities=Capabilities(allow_network=True))
    assert [row.id for row in report_fast.rows] == ["checks_repo_alpha_contract"]


def test_report_envelope_and_text_renderer_stability(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr("atlasctl.checks.runner.list_checks", lambda: ())
    report = run_checks(
        CheckContext(repo_root=tmp_path.resolve(), fs=FS(repo_root=tmp_path.resolve(), allowed_roots=("artifacts/evidence",))),
        selections=CheckSelector(patterns=("checks_repo_never_matches",)),
    )
    payload = build_report_payload(report, run_id="run-stable")
    assert payload["schema_name"] == "atlasctl.check-run.v1"
    assert payload["run_id"] == "run-stable"
    text = render_text(payload, quiet=False, verbose=False)
    assert "summary:" in text


def test_registry_invariants_and_aliases() -> None:
    checks = list_checks()
    ids = [str(check.check_id) for check in checks]
    assert len(ids) == len(set(ids))
    for check in checks:
        if str(check.check_id).startswith("checks_"):
            assert check.owners
            assert check.effects
            assert str(check.result_code)
            assert str(check.check_id).split("_", 2)[1] == str(check.domain)
        assert all(str(tag).strip() for tag in check.tags)
        assert all(str(tag) in set(map(str, TAGS_VOCAB)) for tag in check.tags)
    aliases = resolve_aliases()
    assert isinstance(aliases, tuple)


def test_select_checks_deterministic_order() -> None:
    checks = [
        CheckDef("checks_repo_zeta_contract", "repo", "zeta", 100, _ok),
        CheckDef("checks_repo_alpha_contract", "repo", "alpha", 100, _ok),
    ]
    selected = select_checks(checks, CheckSelector(patterns=("checks_repo_*",)))
    assert [c.check_id for c in selected] == ["checks_repo_alpha_contract", "checks_repo_zeta_contract"]
