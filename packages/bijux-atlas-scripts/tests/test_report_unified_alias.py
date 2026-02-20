from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_report_unified_alias_collects() -> None:
    run_id = "report-unified-alias"
    lane_dir = ROOT / "artifacts/evidence/make/lane-docs" / run_id
    lane_dir.mkdir(parents=True, exist_ok=True)
    lane_payload = {
        "schema_version": 1,
        "report_version": 1,
        "lane": "lane-docs",
        "run_id": run_id,
        "status": "pass",
        "started_at": "2026-02-20T00:00:00Z",
        "ended_at": "2026-02-20T00:00:01Z",
        "duration_seconds": 1.0,
        "log": "artifacts/isolate/lane-docs/log.txt",
        "artifact_paths": [],
    }
    (lane_dir / "report.json").write_text(json.dumps(lane_payload), encoding="utf-8")

    proc = _run_cli("--quiet", "report", "unified", "--run-id", run_id)
    assert proc.returncode == 0, proc.stderr
    out = ROOT / "artifacts/evidence/make" / run_id / "unified.json"
    assert out.exists()
