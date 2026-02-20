from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def run_legacy_script(script_path: str, args: list[str]) -> int:
    root = repo_root()
    script = (root / script_path).resolve()
    if not script.exists():
        raise SystemExit(f"script not found: {script_path}")
    cmd = [sys.executable, str(script), *args]
    proc = subprocess.run(cmd, cwd=root)
    return proc.returncode
