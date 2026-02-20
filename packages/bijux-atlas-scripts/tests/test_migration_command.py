from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", "--json", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_migration_status_json_shape() -> None:
    proc = _run("migration", "status")
    assert proc.returncode in (0, 3), proc.stderr
    payload = json.loads(proc.stdout)
    assert "total_legacy_scripts" in payload
    assert "remaining" in payload
    assert "blocked" in payload


def test_migration_diff_runs() -> None:
    proc = _run("migration", "diff")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert "removed_since_last" in payload
    assert "added_since_last" in payload


def test_migration_gate_accepts_allow_override() -> None:
    proc = _run("migration", "gate", "--allow-remaining", "999999")
    assert proc.returncode in (0, 3), proc.stderr
    payload = json.loads(proc.stdout)
    assert int(payload["allow_remaining"]) == 999999
