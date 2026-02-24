from __future__ import annotations

import json
from pathlib import Path

from atlasctl.checks.adapters import FS
from atlasctl.checks.model import CheckContext, CheckDef
from atlasctl.checks.policy import Capabilities
from atlasctl.checks.report import build_report_payload
from atlasctl.checks.runner import run_checks
from atlasctl.contracts.validate import validate
from tests.helpers import golden_text


def _ok(_repo_root: Path) -> tuple[int, list[str]]:
    return 0, []


def _fail(_repo_root: Path) -> tuple[int, list[str]]:
    return 1, ["boom"]


def _normalized(payload: dict[str, object]) -> dict[str, object]:
    out = json.loads(json.dumps(payload))
    out["run_id"] = ""
    summary = out.get("summary", {})
    if isinstance(summary, dict):
        summary["duration_ms"] = 0
    for row in out.get("rows", []):
        if isinstance(row, dict):
            row["duration_ms"] = 0
            metrics = row.get("metrics", {})
            if isinstance(metrics, dict):
                metrics["duration_ms"] = 0
    return out


def test_runner_fail_fast_matches_golden(monkeypatch, tmp_path: Path) -> None:
    checks = (
        CheckDef("checks_repo_alpha_contract", "repo", "alpha", 100, _fail, effects=("fs_read",), owners=("platform",)),
        CheckDef("checks_repo_beta_contract", "repo", "beta", 100, _ok, effects=("fs_read",), owners=("platform",)),
    )
    monkeypatch.setattr("atlasctl.checks.runner.list_checks", lambda: checks)
    ctx = CheckContext(repo_root=tmp_path.resolve(), fs=FS(repo_root=tmp_path.resolve(), allowed_roots=("artifacts/evidence",)))
    report = run_checks(ctx, fail_fast=True, capabilities=Capabilities(allow_network=True))
    payload = _normalized(build_report_payload(report, run_id="golden"))
    schema_name = payload.get("schema_name")
    if isinstance(schema_name, str) and schema_name:
        validate(schema_name, payload)
    assert json.dumps(payload, sort_keys=True) == golden_text("check/runner-fail-fast.json.golden")


def test_runner_effect_denied_skip_matches_golden(monkeypatch, tmp_path: Path) -> None:
    checks = (
        CheckDef("checks_repo_alpha_contract", "repo", "alpha", 100, _ok, effects=("fs_read",), owners=("platform",)),
        CheckDef("checks_repo_beta_contract", "repo", "beta", 100, _ok, effects=("network",), owners=("platform",)),
    )
    monkeypatch.setattr("atlasctl.checks.runner.list_checks", lambda: checks)
    ctx = CheckContext(repo_root=tmp_path.resolve(), fs=FS(repo_root=tmp_path.resolve(), allowed_roots=("artifacts/evidence",)))
    report = run_checks(ctx, fail_fast=False, capabilities=Capabilities(allow_network=False))
    payload = _normalized(build_report_payload(report, run_id="golden"))
    schema_name = payload.get("schema_name")
    if isinstance(schema_name, str) and schema_name:
        validate(schema_name, payload)
    assert json.dumps(payload, sort_keys=True) == golden_text("check/runner-effect-denied-skip.json.golden")
