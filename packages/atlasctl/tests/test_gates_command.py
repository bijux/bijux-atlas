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
