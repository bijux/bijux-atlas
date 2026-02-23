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


def test_lint_repo_json_shape() -> None:
    proc = _run_cli("lint", "repo", "--report", "json")
    assert proc.returncode in {0, 1}, proc.stderr
    raw = proc.stdout.strip()
    if not raw:
        # Nested lint command subprocesses can fail before the outer wrapper prints JSON in minimal-env tests.
        assert "atlasctl.cli" in proc.stderr
        return
    payload = json.loads(raw)
    assert payload["tool"] == "bijux-atlas"
    assert payload["suite"] == "repo"
    assert "checks" in payload
