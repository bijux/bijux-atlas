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


def test_legacy_audit_json() -> None:
    proc = _run_cli("--quiet", "legacy", "audit", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["action"] == "audit"
    assert payload["status"] == "pass"
    assert isinstance(payload["references"], list)


def test_legacy_check_json() -> None:
    proc = _run_cli("--quiet", "legacy", "check", "--report", "json")
    assert proc.returncode in (0, 1), proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["action"] == "check"
    assert "uncovered_count" in payload
