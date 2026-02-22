from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[5]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    return subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", "--quiet", "--json", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_registry_checks_reads_catalog() -> None:
    proc = _run_cli("registry", "checks")
    assert proc.returncode == 0, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["kind"] == "atlasctl-checks-catalog"
    assert isinstance(payload["checks"], list)
    assert payload["checks"][0]["id"].startswith("checks_")


def test_registry_checks_index_check_mode() -> None:
    proc = _run_cli("registry", "checks-index", "--check")
    assert proc.returncode == 0, proc.stderr
