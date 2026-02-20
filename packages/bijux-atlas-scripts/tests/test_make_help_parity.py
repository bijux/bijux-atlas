from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _run(*args: str) -> str:
    env = {"PYTHONPATH": str(ROOT / "packages/bijux-atlas-scripts/src")}
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


def test_help_renderer_parity_modes() -> None:
    modes = [None, "gates", "list", "advanced", "all"]
    for mode in modes:
        old_args = ["packages/bijux-atlas-scripts/src/bijux_atlas_scripts/layout_checks/render_public_help.py"]
        new_args = [
            "-m",
            "bijux_atlas_scripts.make.help",
        ]
        if mode:
            old_args += ["--mode", mode]
            new_args += ["--mode", mode]
        old = _run(*old_args)
        new = _run(*new_args)
        assert old == new
