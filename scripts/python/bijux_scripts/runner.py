from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def run_python(script_rel: str, *args: str) -> int:
    env = os.environ.copy()
    extra = str(ROOT / "scripts/python")
    env["PYTHONPATH"] = f"{extra}:{env.get('PYTHONPATH', '')}" if env.get("PYTHONPATH") else extra
    proc = subprocess.run([sys.executable, str(ROOT / script_rel), *args], env=env)
    return proc.returncode
