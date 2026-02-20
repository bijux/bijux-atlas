from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def test_cli_help_snapshot() -> None:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    proc = subprocess.run(
        [sys.executable, "-m", "atlasctl.cli", "--help"],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    golden = (Path(__file__).resolve().parent / "goldens" / "cli_help_snapshot.txt").read_text(encoding="utf-8")
    assert proc.stdout == golden
