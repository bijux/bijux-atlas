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


def test_ports_show_json() -> None:
    proc = _run_cli("--quiet", "ports", "show", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
    assert payload["area"] == "ports"
    assert "ports" in payload["details"]


def test_ports_reserve_writes_artifacts(tmp_path: Path) -> None:
    proc = _run_cli(
        "--quiet",
        "--run-id",
        "t-ports",
        "--evidence-root",
        str(tmp_path),
        "ports",
        "reserve",
        "--name",
        "test-http",
        "--report",
        "json",
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["details"]["name"] == "test-http"
    report = tmp_path / "ports" / "t-ports" / "report.json"
    assert report.exists()


def test_cleanup_older_than(tmp_path: Path) -> None:
    stale = tmp_path / "old-area" / "run-old"
    stale.mkdir(parents=True, exist_ok=True)
    (stale / "x.log").write_text("x\n", encoding="utf-8")
    proc = _run_cli(
        "--quiet",
        "--run-id",
        "t-clean",
        "--evidence-root",
        str(tmp_path),
        "cleanup",
        "--older-than",
        "0",
        "--report",
        "json",
    )
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["status"] == "pass"
    assert payload["area"] == "cleanup"
