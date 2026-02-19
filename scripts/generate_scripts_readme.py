#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
cmd = [sys.executable, str(ROOT / "scripts/gen/generate_scripts_readme.py"), *sys.argv[1:]]
raise SystemExit(subprocess.run(cmd).returncode)
