from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", "--json", "check", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_check_all_json_shape() -> None:
    proc = _run("all")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "checks-runner"
    assert payload["domain"] == "all"
    assert payload["total_count"] >= 10
    first = payload["checks"][0]
    for key in ("id", "domain", "status", "duration_ms", "budget_ms", "budget_status", "errors"):
        assert key in first


def test_check_domain_make_runs() -> None:
    proc = _run("domain", "make")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["domain"] == "make"
