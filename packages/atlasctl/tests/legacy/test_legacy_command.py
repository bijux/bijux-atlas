from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

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


def test_legacy_inventory_json() -> None:
    proc = _run_cli("--quiet", "legacy", "inventory", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["action"] == "inventory"
    assert payload["status"] == "ok"
    assert isinstance(payload["legacy_modules"], list)


def test_no_legacy_package_exists() -> None:
    legacy = ROOT / "packages/atlasctl/src/atlasctl/legacy"
    assert not legacy.exists()
