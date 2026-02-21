from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_deps_export_requirements() -> None:
    proc = _run("deps", "export-requirements")
    assert proc.returncode == 0, proc.stderr
    assert "requirements.lock.txt" in proc.stdout


def test_deps_cold_start_budget() -> None:
    proc = _run("deps", "cold-start", "--runs", "1", "--max-ms", "5000")
    assert proc.returncode == 0, proc.stderr
    assert "cold-start-ms" in proc.stdout
