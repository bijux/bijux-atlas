from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[4]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["PYTHONPATH"] = str(ROOT / "packages/atlasctl/src")
    extra: list[str] = []
    if os.environ.get("BIJUX_SCRIPTS_TEST_NO_NETWORK") == "1":
        extra.append("--no-network")
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *extra, *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_report_collect_and_validate_and_summarize() -> None:
    run_id = "report-test-run"
    lane_dir = ROOT / "artifacts/evidence/make/lane-scripts" / run_id
    lane_dir.mkdir(parents=True, exist_ok=True)
    lane_payload = {
        "schema_version": 1,
        "report_version": 1,
        "lane": "lane-scripts",
        "run_id": run_id,
        "status": "pass",
        "started_at": "2026-02-20T00:00:00Z",
        "ended_at": "2026-02-20T00:00:10Z",
        "duration_seconds": 10.0,
        "log": "artifacts/isolate/lane-scripts/log.txt",
        "artifact_paths": [],
    }
    (lane_dir / "report.json").write_text(json.dumps(lane_payload), encoding="utf-8")

    collect = _run_cli("report", "collect", "--run-id", run_id)
    assert collect.returncode == 0, collect.stderr
    unified_path = ROOT / "artifacts/evidence/make" / run_id / "unified.json"
    payload = json.loads(unified_path.read_text(encoding="utf-8"))
    schema = json.loads((ROOT / "ops/schema/report/unified.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    assert payload["report_version"] == 1
    assert payload["bypass_debt"]["status"] in {"pass", "warn", "fail"}
    assert isinstance(payload["bypass_debt"]["entry_count"], int)

    validate = _run_cli("report", "validate", "--run-id", run_id)
    assert validate.returncode == 0, validate.stderr
    assert "ok" in validate.stdout

    summarize = _run_cli("report", "summarize", "--run-id", run_id)
    assert summarize.returncode == 0, summarize.stderr
    summary_md = ROOT / "artifacts/evidence/make" / run_id / "summary.md"
    assert summary_md.exists()


def test_report_diff_and_export() -> None:
    run_a = "report-diff-a"
    run_b = "report-diff-b"
    for run_id, status in ((run_a, "pass"), (run_b, "fail")):
        lane_dir = ROOT / "artifacts/evidence/make/lane-docs" / run_id
        lane_dir.mkdir(parents=True, exist_ok=True)
        payload = {
            "schema_version": 1,
            "report_version": 1,
            "lane": "lane-docs",
            "run_id": run_id,
            "status": status,
            "started_at": "2026-02-20T00:00:00Z",
            "ended_at": "2026-02-20T00:00:10Z",
            "duration_seconds": 10.0,
            "log": "artifacts/isolate/lane-docs/log.txt",
            "artifact_paths": [],
        }
        (lane_dir / "report.json").write_text(json.dumps(payload), encoding="utf-8")
        assert _run_cli("report", "collect", "--run-id", run_id).returncode == 0

    diff = _run_cli("report", "diff", "--from-run", run_a, "--to-run", run_b)
    assert diff.returncode == 0, diff.stderr
    assert "lane-docs: pass -> fail" in diff.stdout

    export = _run_cli("report", "export", "--run-id", run_b)
    assert export.returncode == 0, export.stderr
    bundle = ROOT / "artifacts/evidence/make" / run_b / "evidence.tar.gz"
    assert bundle.exists()


def test_report_budgets_json_by_domain() -> None:
    proc = _run_cli("report", "budgets", "--json", "--by-domain")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert "by_domain" in payload


def test_reporting_ci_summary_latest() -> None:
    run_id = "ci-summary-test"
    ci_dir = ROOT / "artifacts/evidence/ci" / run_id
    ci_dir.mkdir(parents=True, exist_ok=True)
    (ci_dir / "suite-ci.report.json").write_text(
        json.dumps({"status": "ok", "summary": {"passed": 2, "failed": 0, "skipped": 0}}) + "\n",
        encoding="utf-8",
    )
    (ci_dir / "suite-ci.summary.txt").write_text("ok\n", encoding="utf-8")
    (ci_dir / "run.meta.json").write_text("{}\n", encoding="utf-8")

    proc = _run_cli("reporting", "ci-summary", "--latest")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    schema = json.loads((ROOT / "configs/contracts/reporting-ci-summary-output.schema.json").read_text(encoding="utf-8"))
    jsonschema.validate(payload, schema)
    assert payload["kind"] == "ci-summary"
    assert payload["suite_status"] == "ok"
    assert payload["suite_summary"]["passed"] == 2
