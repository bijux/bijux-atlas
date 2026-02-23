from __future__ import annotations

import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[5]


def _run(*argv: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["./bin/atlasctl", *argv],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def test_ops_gen_index_requires_acknowledgement() -> None:
    proc = _run("ops", "gen", "--report", "json", "index")
    assert proc.returncode != 0
    assert "--i-know-what-im-doing" in (proc.stdout + proc.stderr)


def test_ops_stack_down_supports_dry_run() -> None:
    proc = _run("--dry-run", "ops", "stack", "--report", "json", "down", "--dry-run")
    assert proc.returncode == 0, proc.stderr
    assert '"dry_run": true' in proc.stdout


def test_ops_clean_generated_supports_dry_run() -> None:
    proc = _run("--dry-run", "ops", "clean-generated", "--dry-run", "--report", "json")
    assert proc.returncode == 0, proc.stderr
    assert '"dry_run": true' in proc.stdout
