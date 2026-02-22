from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def _run(*args: str) -> str:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    proc = subprocess.run(
        [sys.executable, *args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    return proc.stdout


def _run_cmd(*args: str) -> str:
    env = {"PYTHONPATH": str(ROOT / "packages/atlasctl/src")}
    proc = subprocess.run(
        [*args],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    return proc.stdout


def test_help_renderer_parity_modes() -> None:
    modes = [None, "gates", "list", "advanced", "all"]
    for mode in modes:
        old_args = ["packages/atlasctl/src/atlasctl/checks/layout/domains/public_surface/render_public_help.py"]
        new_args = [
            "-m",
            "atlasctl.make.help",
        ]
        if mode:
            old_args += ["--mode", mode]
            new_args += ["--mode", mode]
        old = _run(*old_args)
        new = _run(*new_args)
        assert old == new


def test_make_help_matches_atlasctl_make_help() -> None:
    make_help = _run_cmd("make", "-s", "help")
    atlasctl_help = _run("-m", "atlasctl.cli", "--quiet", "make", "help")
    assert make_help == atlasctl_help
