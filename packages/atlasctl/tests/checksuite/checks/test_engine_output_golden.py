from __future__ import annotations

from pathlib import Path

from atlasctl.checks.adapters import FS
from atlasctl.checks.engine import EngineOptions, run_checks
from atlasctl.checks.model import CheckContext, CheckDef
from atlasctl.checks.report import build_report_payload, render_json


def test_engine_json_output_golden(tmp_path: Path) -> None:
    def ok(_root: Path):
        return 0, []

    ctx = CheckContext(repo_root=tmp_path, fs=FS(repo_root=tmp_path, allowed_roots=("artifacts/evidence/",)), env={})
    check = CheckDef(
        "checks_repo_engine_output",
        "repo",
        "engine output",
        100,
        ok,
        tags=("repo",),
        owners=("platform",),
        effects=("fs_read",),
    )
    report = run_checks((check,), None, ctx, options=EngineOptions(only_fast=False, include_slow=True))
    payload = build_report_payload(report, run_id="run-engine", tool="atlasctl")
    actual = render_json(payload) + "\n"
    expected = Path("packages/atlasctl/tests/goldens/check/engine-run.json.golden").read_text(encoding="utf-8")
    assert actual == expected
