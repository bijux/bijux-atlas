from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def _run(*argv: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["./bin/atlasctl", *argv], cwd=ROOT, text=True, capture_output=True, check=False)


def test_ops_stack_cleanup_dry_run_json() -> None:
    proc = _run("--dry-run", "ops", "stack", "--report", "json", "cleanup", "--dry-run")
    assert proc.returncode == 0, proc.stderr
    assert '"dry_run": true' in proc.stdout


def test_ops_stack_reset_dry_run_json() -> None:
    proc = _run("--dry-run", "ops", "stack", "--report", "json", "reset", "--dry-run")
    assert proc.returncode == 0, proc.stderr
    assert '"dry_run": true' in proc.stdout

