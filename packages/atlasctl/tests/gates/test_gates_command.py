from __future__ import annotations

import json
import subprocess
import sys
import argparse
from pathlib import Path

from atlasctl.core.context import RunContext
from atlasctl.gates import command as gates_command

ROOT = Path(__file__).resolve().parents[4]


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


def test_gates_list_json() -> None:
    proc = _run_cli("--quiet", "gates", "list", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
    assert any(lane["id"] == "lane-cargo" for lane in payload["lanes"])


def test_gates_run_unknown_lane_fails() -> None:
    proc = _run_cli("--quiet", "gates", "run", "--lane", "lane-missing", "--report", "json")
    assert proc.returncode == 2
    payload = json.loads(proc.stdout)
    assert payload["status"] == "fail"


def test_gates_run_writes_artifacts_and_accepts_positional_lane(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.setattr(
        gates_command,
        "_load_lanes",
        lambda _repo_root: (
            {"lane-a": gates_command.Lane("lane-a", "lane a", "ci")},
            {"root": ["lane-a"]},
        ),
    )
    monkeypatch.setattr(
        gates_command,
        "_run_one",
        lambda _repo_root, lane: {"id": lane.lane_id, "make_target": lane.make_target, "status": "pass"},
    )
    ctx = RunContext.from_args("gates-test", None, "test", False)
    ns = argparse.Namespace(gates_cmd="run", lane_id="lane-a", lane="", preset="root", all=False, parallel=False, jobs=1, report="json")
    rc = gates_command.run_gates_command(ctx, ns)
    assert rc == 0
    report_json = ctx.evidence_root / "gates" / ctx.run_id / "report.json"
    report_txt = ctx.evidence_root / "gates" / ctx.run_id / "report.txt"
    assert report_json.exists()
    assert report_txt.exists()
