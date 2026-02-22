from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_clean_command_only_targets_scripts_artifacts() -> None:
    proc = _run_cli("--quiet", "clean", "--json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
    assert payload["action"] == "clean"


def test_report_pr_summary_and_index_and_gc() -> None:
    run_id = "artifact-cmd-run"
    lane_dir = ROOT / "artifacts/evidence/make/lane-scripts" / run_id
    lane_dir.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema_version": 1,
        "report_version": 1,
        "lane": "lane-scripts",
        "run_id": run_id,
        "status": "pass",
        "started_at": "2026-02-20T00:00:00Z",
        "ended_at": "2026-02-20T00:00:01Z",
        "duration_seconds": 1.0,
        "log": "artifacts/isolate/lane-scripts/log.txt",
        "artifact_paths": [],
    }
    (lane_dir / "report.json").write_text(json.dumps(payload), encoding="utf-8")

    collect = _run_cli("--quiet", "report", "collect", "--run-id", run_id)
    assert collect.returncode == 0, collect.stderr

    pr = _run_cli("--quiet", "report", "pr-summary", "--run-id", run_id)
    assert pr.returncode == 0, pr.stderr

    idx = _run_cli("--quiet", "report", "artifact-index", "--limit", "5")
    assert idx.returncode == 0, idx.stderr
    index_payload = json.loads(idx.stdout)
    assert "artifact_runs" in index_payload

    gc = _run_cli("--quiet", "report", "artifact-gc", "--older-than-days", "0")
    assert gc.returncode == 0, gc.stderr
