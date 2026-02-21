from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def test_repo_stats_command_json_payload() -> None:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", "--quiet", "--format", "json", "repo", "stats", "--top", "5"],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlasctl"
    assert payload["command"] == "repo stats"
    assert len(payload["summary"]["densest_by_py_files"]) == 5
