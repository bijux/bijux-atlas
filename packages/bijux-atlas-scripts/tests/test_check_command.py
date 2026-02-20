from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
    return subprocess.run(
        [sys.executable, "-m", "bijux_atlas_scripts.cli", "--format", "json", *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_check_make_uses_lint_payload() -> None:
    proc = _run_cli("check", "make")
    assert proc.returncode in {0, 1}, proc.stderr
    payload = json.loads(proc.stdout)
    assert payload["tool"] == "bijux-atlas"
    assert payload["suite"] == "makefiles"
    assert "checks" in payload

