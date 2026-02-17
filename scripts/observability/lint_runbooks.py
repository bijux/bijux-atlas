#!/usr/bin/env python3
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
cmd = [sys.executable, str(ROOT / "scripts" / "docs" / "check_runbooks_contract.py")]
raise SystemExit(subprocess.run(cmd, cwd=ROOT).returncode)
