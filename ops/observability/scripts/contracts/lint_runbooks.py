#!/usr/bin/env python3
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
cmd = [sys.executable, str(ROOT / "scripts" / "docs" / "check_runbooks_contract.py")]
raise SystemExit(subprocess.run(cmd, cwd=ROOT).returncode)