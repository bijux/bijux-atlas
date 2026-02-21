from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def test_version_works_outside_git() -> None:
    with tempfile.TemporaryDirectory(prefix="atlasctl-outside-git-") as td:
        cwd = Path(td)
        env = os.environ.copy()
        env["PYTHONPATH"] = str(ROOT / "packages/atlasctl/src")
        proc = subprocess.run(
            [sys.executable, "-m", "atlasctl.cli", "--version"],
            cwd=cwd,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        assert proc.returncode == 0
        assert "atlasctl 0.1.0+unknown" in proc.stdout


def test_python_module_entrypoint_works_outside_git() -> None:
    with tempfile.TemporaryDirectory(prefix="atlasctl-outside-git-") as td:
        cwd = Path(td)
        env = os.environ.copy()
        env["PYTHONPATH"] = str(ROOT / "packages/atlasctl/src")
        proc = subprocess.run(
            [sys.executable, "-m", "atlasctl", "--version"],
            cwd=cwd,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        assert proc.returncode == 0
        assert "atlasctl 0.1.0+unknown" in proc.stdout
