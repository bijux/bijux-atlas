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


def test_lint_ops_alias_json_shape() -> None:
    proc = _run_cli("--json", "lint", "ops", "--fail-fast")
    assert proc.returncode in {0, 1}, proc.stderr
    raw = proc.stdout.strip()
    payload = json.loads(raw)
    assert payload["tool"] == "atlasctl"
    assert payload["kind"] == "check-run"
    assert isinstance(payload.get("rows"), list)
    assert all(row.get("category") in {"lint", "check"} for row in payload.get("rows", []))
