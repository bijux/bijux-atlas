from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "packages/bijux-atlas-scripts/src"


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["PYTHONPATH"] = str(SRC)
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_help() -> None:
    proc = _run("--help")
    assert proc.returncode == 0
    assert "atlas-scripts" in proc.stdout


def test_doctor_json() -> None:
    proc = _run("doctor", "--json")
    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "atlas-scripts"


def test_gates_list() -> None:
    proc = _run("gates", "list")
    assert proc.returncode == 0
    payload = json.loads(proc.stdout)
    assert "scripts-check" in payload["gates"]
